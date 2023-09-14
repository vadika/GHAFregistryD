use warp::Filter;
use serde::{Deserialize, Serialize};
use redis::{Client, Commands};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VM {
    name: String,
    vm_type: VMType,
    addresses: Addresses,
    xdg_run: Option<String>,
    mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VMType {
    system_app: SystemAppType,
    run_type: RunType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum SystemAppType {
    System,
    App,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum RunType {
    LongRun,
    OneShot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Addresses {
    ip: String,
    vsock: String,
}

#[tokio::main]
async fn main() {
    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and_then(register_vm);

    let run = warp::post()
        .and(warp::path("run"))
        .and(warp::path::param())
        .and_then(run_vm);

    let connect = warp::post()
        .and(warp::path("connect"))
        .and(warp::path::param())
        .and_then(connect_vm);

    let stop = warp::post()
        .and(warp::path("stop"))
        .and(warp::path::param())
        .and_then(stop_vm);

    let get_status = warp::get()
        .and(warp::path("status"))
        .and(warp::path::param())
        .and_then(get_vm_status);

    let unregister = warp::delete()
        .and(warp::path("unregister"))
        .and(warp::path::param())
        .and_then(unregister_vm);

    let list = warp::get()
        .and(warp::path("list"))
        .and_then(list_vms);

    let routes = register
        .or(run)
        .or(connect)
        .or(stop)
        .or(get_status)
        .or(unregister)
        .or(list);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn register_vm(vm: VM) -> Result<impl warp::Reply, warp::Rejection> {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = con.set(&vm.name, serde_json::to_string(&vm).unwrap()).unwrap();
    Ok(warp::reply::json(&vm))
}

async fn run_vm(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Running VM with name: {}", name);
    Ok(warp::reply::with_status("VM started.", warp::http::StatusCode::OK))
}

async fn connect_vm(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Connecting to VM with name: {}", name);
    Ok(warp::reply::with_status("Connected to VM.", warp::http::StatusCode::OK))
}

async fn stop_vm(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Stopping VM with name: {}", name);
    Ok(warp::reply::with_status("VM stopped.", warp::http::StatusCode::OK))
}

async fn get_vm_status(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    // Sample status for the sake of the example
    let status = format!("VM {} is running.", name);
    Ok(warp::reply::with_status(status, warp::http::StatusCode::OK))
}

async fn unregister_vm(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = con.del(&name).unwrap();
    Ok(warp::reply::with_status("VM unregistered.", warp::http::StatusCode::OK))
}

async fn list_vms() -> Result<impl warp::Reply, warp::Rejection> {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let vm_names: Vec<String> = con.keys("*").unwrap();
    let mut vms = Vec::new();
    for name in vm_names {
        let vm_data: String = con.get(&name).unwrap();
        let vm: VM = serde_json::from_str(&vm_data).unwrap();
        vms.push(vm);
    }
    Ok(warp::reply::json(&vms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::test::request;

    // Utility to clear the test Redis database
    async fn clear_redis() {
        let client = Client::open("redis://127.0.0.1:6379/").unwrap();
        let mut con = client.get_connection().unwrap();
        let _: () = con.flushdb().unwrap();
    }

    #[tokio::test]
    async fn test_register_vm() {
        clear_redis().await;

        let vm = VM {
            name: "test_vm".to_string(),
            vm_type: VMType {
                system_app: SystemAppType::System,
                run_type: RunType::LongRun,
            },
            addresses: Addresses {
                ip: "127.0.0.1".to_string(),
                vsock: "vsock_value".to_string(),
            },
            xdg_run: Some("xdg_value".to_string()),
            mime_type: Some("mime_value".to_string()),
        };

        let response = request()
            .method("POST")
            .path("/register")
            .json(&vm)
            .reply(&register_vm)
            .await;

        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_run_vm() {
        clear_redis().await;

        // First, we register a VM to run it
        let vm = VM {
            name: "run_test_vm".to_string(),
            vm_type: VMType {
                system_app: SystemAppType::System,
                run_type: RunType::LongRun,
            },
            addresses: Addresses {
                ip: "127.0.0.1".to_string(),
                vsock: "vsock_value".to_string(),
            },
            xdg_run: None,
            mime_type: None,
        };

        request()
            .method("POST")
            .path("/register")
            .json(&vm)
            .reply(&register_vm)
            .await;

        let response = request()
            .method("POST")
            .path("/run/run_test_vm")
            .reply(&run_vm)
            .await;

        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_list_vms() {
        clear_redis().await;

        let response = request()
            .method("GET")
            .path("/list")
            .reply(&list_vms)
            .await;

        assert_eq!(response.status(), 200);
    }

    // Add tests for other routes...
}

