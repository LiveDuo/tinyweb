
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::env::temp_dir;
use std::path::PathBuf;

use fantoccini::wd::Capabilities;
use fantoccini::{ClientBuilder, Locator};

pub const WASM_TRIPLET: &str = "wasm32-unknown-unknown";

fn get_pid_on_port(port: u16) -> Option<u32> {
    let output = Command::new("lsof").args(&["-ti", format!(":{port}").as_str()]).output().unwrap();
    let stdout_opt = if output.stdout.is_empty() { None } else { Some(output.stdout) };
    stdout_opt.map(|o| std::str::from_utf8(&o).map(|p| p.trim().parse().unwrap()).unwrap())
}

fn setup_temp_project() -> PathBuf {
    // get paths
    let temp_dir = temp_dir();
    let cwd = std::env::current_dir().unwrap();
    let project_path = cwd.parent().unwrap().parent().unwrap();

    // build wasm
    let p = Command::new("cargo").args(["build", "-p", "minimal", "--target", WASM_TRIPLET]).output().unwrap();
    assert!(p.status.success());

    // copy wasm
    let wasm_path = project_path.join("target").join(WASM_TRIPLET).join("debug").join("minimal.wasm");
    std::fs::copy(wasm_path, temp_dir.join("client.wasm")).unwrap();

    // copy js
    let js_path = project_path.join("src").join("js").join("main.js");
    std::fs::copy(js_path, temp_dir.join("main.js")).unwrap();

    // copy html
    let html = r#"<script src="main.js"></script><script type="application/wasm" src="client.wasm"></script>"#;
    std::fs::write(temp_dir.join("index.html"), html).unwrap();

    temp_dir
}

// lsof -i tcp:4444 && kill -9 ${PID}
#[tokio::test]
async fn test_wasm() -> Result<(), fantoccini::error::CmdError> {

    // start daemon
    let lock = Arc::new(Mutex::new(None));
    let lock_clone = lock.clone();
    std::thread::spawn(move || {

        let pid = get_pid_on_port(4444);
        if let Some(pid) = pid {
            Command::new("kill").arg(format!("{}", pid)).status().unwrap();
        }

        let child = Command::new("geckodriver").stderr(Stdio::null()).spawn().unwrap();
        lock_clone.lock().map(|mut s| { *s = Some(child); }).unwrap();
    });
    std::thread::sleep(Duration::from_millis(1_000));
    
    // open browser
    let mut client_builder = ClientBuilder::native();
    let mut caps = Capabilities::new();
    caps.insert("moz:firefoxOptions".to_string(), serde_json::json!({ "args": ["--headless"] }));
    client_builder.capabilities(caps);
    let client = client_builder.connect("http://localhost:4444").await.unwrap();

    // prepare project
    let project_dir = setup_temp_project();
    
    // load html
    let index_html = "/index.html";
    let url = format!("file://{}{}", project_dir.to_str().unwrap(), index_html);
    client.goto(&url).await?;
    
    std::thread::sleep(Duration::from_millis(1_000));

    // check body
    let body = client.find(Locator::Css("body")).await?;
    let body_str = body.html(true).await?;
    assert!(body_str.contains("hello"));

    // stop browser
    client.close().await?;

    // stop daemon
    lock.lock().map(|mut s| {
        let child = s.as_mut().unwrap();
        child.kill().unwrap();
    }).unwrap();

    Ok(())

}