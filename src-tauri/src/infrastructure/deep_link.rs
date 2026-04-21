//! 解析启动参数中的 `oclive://install?...`（系统协议注册后由 OS 传入 argv）。

use once_cell::sync::Lazy;
use parking_lot::Mutex;

static PENDING_INSTALL_GIT_URLS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn decode_url_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = bytes[i + 1];
                let lo = bytes[i + 2];
                let hex = [hi, lo];
                if let Ok(s) = std::str::from_utf8(&hex) {
                    if let Ok(v) = u8::from_str_radix(s, 16) {
                        out.push(v as char);
                        i += 3;
                        continue;
                    }
                }
                out.push('%');
                i += 1;
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    out
}

fn parse_install_git_url(url: &str) -> Option<String> {
    let t = url.trim();
    if !t.starts_with("oclive://install") {
        return None;
    }
    let query = t.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("").trim();
        let v = it.next().unwrap_or("").trim();
        if k.eq_ignore_ascii_case("plugin") {
            let decoded = decode_url_component(v).trim().to_string();
            if !decoded.is_empty() {
                return Some(decoded);
            }
        }
    }
    None
}

pub fn seed_pending_install_urls_from_args<I>(args: I)
where
    I: IntoIterator<Item = String>,
{
    let mut q = PENDING_INSTALL_GIT_URLS.lock();
    for arg in args {
        if let Some(git_url) = parse_install_git_url(&arg) {
            q.push(git_url);
        }
    }
}

pub fn take_pending_install_git_urls() -> Vec<String> {
    let mut q = PENDING_INSTALL_GIT_URLS.lock();
    std::mem::take(&mut *q)
}
