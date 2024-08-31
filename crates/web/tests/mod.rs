
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fantoccini::wd::Capabilities;
use fantoccini::{ClientBuilder, Locator};

fn get_pid_on_port(port: u16) -> Option<u32> {
    let output = Command::new("lsof").args(&["-ti", format!(":{port}").as_str()]).output().unwrap();

    if !output.stdout.is_empty() {
        let pid_str = std::str::from_utf8(&output.stdout).expect("Failed to parse output");
        pid_str.trim().parse().ok()
    } else {
        None
    }
}

fn kill_process(pid: u32) -> Result<(), std::io::Error> {
    Command::new("kill").arg(format!("{}", pid)).status().map(|_| ())
}

// lsof -i tcp:4444 && kill -9 ${PID}
#[tokio::test]
async fn main() -> Result<(), fantoccini::error::CmdError> {

    // start daemon
    let lock = Arc::new(Mutex::new(None));
    let lock_clone = lock.clone();
    std::thread::spawn(move || {

        let pid = get_pid_on_port(4444);
        if let Some(pid) = pid {
            kill_process(pid).unwrap();
        }

        let child = Command::new("geckodriver").stderr(Stdio::null()).spawn().unwrap();
        lock_clone.lock().map(|mut s| { *s = Some(child); }).unwrap();
    });
    std::thread::sleep(Duration::from_millis(100));
    
    // open browser
    let mut client_builder = ClientBuilder::native();
    let mut caps = Capabilities::new();
    caps.insert("moz:firefoxOptions".to_string(), serde_json::json!({ "args": ["--headless"] }));
    client_builder.capabilities(caps);
    let client = client_builder.connect("http://localhost:4444").await.unwrap();

    // load html
    let cwd = std::env::current_dir().unwrap();
    let index_html = "/public/index.html";
    let url = format!("file://{}{}", cwd.to_str().unwrap(), index_html);
    dbg!(&url);
    client.goto(&url).await?;

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