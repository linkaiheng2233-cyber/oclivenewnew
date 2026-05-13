//! 目录插件集成测试用最小 HTTP 侧车：stdout 打印 `OCLIVE_READY <url>` 后对 `/rpc` POST 返回 JSON-RPC 2.0 `result`。
#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    let port = listener.local_addr().expect("local_addr").port();
    println!("OCLIVE_READY http://127.0.0.1:{}/rpc", port);
    std::io::stdout().flush().ok();

    for stream in listener.incoming().flatten() {
        thread::spawn(move || handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0u8; 16_384];
    let n = match stream.read(&mut buf) {
        Ok(0) | Err(_) => return,
        Ok(n) => n,
    };
    let raw = String::from_utf8_lossy(&buf[..n]);
    let body = raw.split("\r\n\r\n").nth(1).map(str::trim).unwrap_or("{}");
    let id = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("id").cloned())
        .unwrap_or(serde_json::json!(0));
    let out = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "p2_stub": true, "method_echo": "ok" }
    });
    let payload = out.to_string();
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );
    let _ = stream.write_all(resp.as_bytes());
}
