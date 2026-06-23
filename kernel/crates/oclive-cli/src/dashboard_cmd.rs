//! `oclive dashboard` — local web dashboard (embedded HTML, default loopback API port).

use crate::registry::load_registry;
use crate::template_catalog::CATALOG;
use anyhow::{Context, Result};
use clap::Parser;
use oclive_kernel_runtime::DEFAULT_API_PORT;
use serde::Serialize;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

#[must_use]
fn default_dashboard_bind() -> String {
    format!("127.0.0.1:{DEFAULT_API_PORT}")
}

#[derive(Parser, Debug)]
pub struct DashboardArgs {
    /// Listen address (default port = [`DEFAULT_API_PORT`])
    #[arg(long, default_value_t = default_dashboard_bind())]
    pub bind: String,
}

#[derive(Serialize)]
struct ProjectRow {
    name: String,
    path: String,
    template: Option<String>,
    created_at: u64,
}

pub fn run(args: DashboardArgs) -> Result<()> {
    let listener = TcpListener::bind(&args.bind).with_context(|| format!("bind {}", args.bind))?;
    eprintln!(
        "oclive dashboard: http://{}/ (Ctrl+C to stop; avoid same port as kernel HTTP API)",
        args.bind
    );
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let _ = handle_client(stream);
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let req = read_request(&mut stream)?;
    let path = req.path.split('?').next().unwrap_or("/");
    let (status, body, ctype) = match path {
        "/" => (200, index_html(), "text/html; charset=utf-8"),
        "/templates" => (200, templates_html(), "text/html; charset=utf-8"),
        "/api/projects" => {
            let rows: Vec<ProjectRow> = load_registry()
                .map(|f| {
                    f.projects
                        .into_iter()
                        .map(|p| ProjectRow {
                            name: p.name,
                            path: p.path,
                            template: p.template,
                            created_at: p.created_at,
                        })
                        .collect()
                })
                .unwrap_or_default();
            (
                200,
                serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into()),
                "application/json",
            )
        }
        p if p.starts_with("/api/project/") => {
            let name = p.trim_start_matches("/api/project/");
            let html = project_detail_html(name);
            (200, html, "text/html; charset=utf-8")
        }
        _ => (404, "not found".into(), "text/plain"),
    };
    let response = format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

struct HttpRequest {
    path: String,
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<HttpRequest> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    let text = String::from_utf8_lossy(&buf[..n]);
    let line = text.lines().next().unwrap_or("GET / ");
    let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
    Ok(HttpRequest { path })
}

fn index_html() -> String {
    include_str!("templates/dashboard/index.html").to_string()
}

fn templates_html() -> String {
    let mut rows = String::new();
    for e in CATALOG {
        rows.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            e.id, e.scene, e.preset, e.description
        ));
    }
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Template library</title>
<style>body{{font-family:system-ui;margin:2rem}}table{{border-collapse:collapse;width:100%}}
td,th{{border:1px solid #ccc;padding:.5rem}}</style></head><body>
<h1>Kernel factory templates</h1><p><a href="/">← Project list</a></p>
<table><tr><th>id</th><th>scene</th><th>preset</th><th>description</th></tr>{rows}</table></body></html>"#
    )
}

fn project_detail_html(name: &str) -> String {
    let entry = load_registry()
        .ok()
        .and_then(|f| f.projects.into_iter().find(|p| p.name == name));
    let Some(entry) = entry else {
        return format!(
            "<html><body><h1>Not found: {name}</h1><a href=\"/\">Back</a></body></html>"
        );
    };
    let root = Path::new(&entry.path);
    let cargo = fs_read_or(&root.join("Cargo.toml"), "(no Cargo.toml)");
    let mono = fs_read_or(&root.join("monolith.toml"), "(Monolith not enabled)");
    let chart = bench_chart_js(root);
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{name}</title>
<style>body{{font-family:system-ui;margin:2rem;max-width:960px}}pre{{background:#f4f4f4;padding:1rem;overflow:auto}}
canvas{{max-width:100%}}</style></head><body>
<h1>{name}</h1><p>Path: <code>{}</code> · Template: {} · <a href="/">List</a></p>
<h2>Cargo.toml</h2><pre>{cargo}</pre>
<h2>monolith.toml</h2><pre>{mono}</pre>
<h2>bench trend (p50 ms)</h2>
<canvas id="c" width="640" height="200"></canvas>
<script>{chart}</script>
</body></html>"#,
        entry.path,
        entry.template.as_deref().unwrap_or("—"),
    )
}

fn fs_read_or(p: &Path, fallback: &str) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|_| fallback.to_string())
}

fn bench_chart_js(root: &Path) -> String {
    let path = root.join("bench_history.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return "document.getElementById('c').insertAdjacentHTML('afterend','<p>No bench_history.json</p>');".into();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return String::new();
    };
    let mut std_pts = Vec::new();
    let mut mono_pts = Vec::new();
    if let Some(arr) = v.get("entries").and_then(|e| e.as_array()) {
        for e in arr {
            if let Some(r) = e.get("report") {
                std_pts.push(r["standard_ms"]["p50"].as_f64().unwrap_or(0.0));
                mono_pts.push(r["monolith_ms"]["p50"].as_f64().unwrap_or(0.0));
            }
        }
    }
    format!(
        r#"
const std={std_pts:?}; const mono={mono_pts:?};
const c=document.getElementById('c'); const ctx=c.getContext('2d');
const w=c.width,h=c.height,p=30;
function draw(data,color) {{
  if(!data.length) return;
  const max=Math.max(...data,1); const min=Math.min(...data);
  ctx.strokeStyle=color; ctx.beginPath();
  data.forEach((v,i)=>{{
    const x=p+(w-2*p)*i/(data.length-1||1);
    const y=h-p-(h-2*p)*(v-min)/(max-min||1);
    i?ctx.lineTo(x,y):ctx.moveTo(x,y);
  }});
  ctx.stroke();
}}
draw(std,'#2563eb'); draw(mono,'#16a34a');
ctx.fillStyle='#333'; ctx.fillText('Standard',p,14); ctx.fillStyle='#16a34a'; ctx.fillText('Monolith',p+50,14);
"#
    )
}
