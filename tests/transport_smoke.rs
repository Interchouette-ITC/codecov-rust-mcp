//! MCP transport smoke: stdio initialize.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn mcp_bin() -> String {
    if let Some(p) = option_env!("CARGO_BIN_EXE_codecov-rust-mcp") {
        return p.to_string();
    }
    let release = env!("CARGO_MANIFEST_DIR").to_string() + "/target/release/codecov-rust-mcp";
    let debug = env!("CARGO_MANIFEST_DIR").to_string() + "/target/debug/codecov-rust-mcp";
    if std::path::Path::new(&release).is_file() {
        release
    } else {
        debug
    }
}

#[test]
fn stdio_initialize() {
    let bin = mcp_bin();
    assert!(
        std::path::Path::new(&bin).is_file(),
        "missing MCP binary at {bin}"
    );
    let mut child = Command::new(&bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mcp stdio");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"stdio-smoke","version":"0.0.1"}}}}}}"#
        )
        .expect("write initialize");
        let _ = stdin.flush();
    }

    let ver = env!("CARGO_PKG_VERSION");
    let stdout = child.stdout.take().expect("stdout");
    let killer_pid = child.id();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        let _ = Command::new("kill").arg(killer_pid.to_string()).status();
    });
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut buf = String::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                buf.push_str(&line);
                if buf.contains("codecov") && buf.contains(ver) {
                    break;
                }
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        buf.contains("codecov"),
        "stdio initialize missing server name: {buf}"
    );
    assert!(
        buf.contains(ver),
        "stdio initialize missing version {ver}: {buf}"
    );
}
