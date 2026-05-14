//! 模板渲染与落盘。

use crate::init::{BackendImpl, ProjectConfig, ProjectType};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;

fn slugify(raw: &str) -> String {
    let mut s: String = raw
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let t = s.trim_matches('-').to_string();
    if t.is_empty() {
        "my-oclive-kernel".into()
    } else {
        t
    }
}

fn be_str(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Builtin => "builtin",
        BackendImpl::Stub => "stub",
        BackendImpl::None => "none",
    }
}

fn template_context(cfg: &ProjectConfig) -> serde_json::Value {
    let safe_package_name = slugify(&cfg.project_name);
    let rust_lib_name = safe_package_name.replace('-', "_");
    json!({
        "project_name": cfg.project_name,
        "safe_package_name": safe_package_name,
        "rust_lib_name": rust_lib_name,
        "project_type": format!("{:?}", cfg.project_type),
        "backend_memory": be_str(cfg.backends.memory),
        "backend_emotion": be_str(cfg.backends.emotion),
        "backend_event": be_str(cfg.backends.event),
        "backend_prompt": be_str(cfg.backends.prompt),
        "backend_llm": be_str(cfg.backends.llm),
        "backend_agent": be_str(cfg.backends.agent),
        "backend_complex_emotion": be_str(cfg.backends.complex_emotion),
        "plugin_directory": cfg.plugins.directory_plugins,
        "plugin_kernel_server": cfg.plugins.kernel_server,
        "plugin_oocp": cfg.plugins.oocp,
        "feature_complex_emotion": cfg.features.use_complex_emotion,
        "with_example_role": cfg.with_example_role,
    })
}

pub fn write_project(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    if out.exists() {
        let mut it = fs::read_dir(out).with_context(|| format!("read_dir {}", out.display()))?;
        if it.next().transpose()?.is_some() {
            anyhow::bail!(
                "输出目录非空：{}。请使用空目录或删除后再试。",
                out.display()
            );
        }
    } else {
        fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    }
    fs::create_dir_all(out.join("src")).context("create src")?;

    let reg = Handlebars::new();
    let ctx = template_context(cfg);

    let cargo_tmpl = match cfg.project_type {
        ProjectType::KernelServer => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/Cargo.kernel.toml.hbs"
            ))
        }
        ProjectType::Library => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/Cargo.library.toml.hbs"
            ))
        }
    };
    let cargo = reg
        .render_template(cargo_tmpl, &ctx)
        .context("render Cargo.toml")?;
    fs::write(out.join("Cargo.toml"), cargo).context("write Cargo.toml")?;

    match cfg.project_type {
        ProjectType::KernelServer => {
            let main_t = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/main.rs.hbs"
            ));
            let main = reg
                .render_template(main_t, &ctx)
                .context("render main.rs")?;
            fs::write(out.join("src").join("main.rs"), main).context("write main.rs")?;
        }
        ProjectType::Library => {
            let lib_t = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/lib.rs.hbs"
            ));
            let lib = reg.render_template(lib_t, &ctx).context("render lib.rs")?;
            fs::write(out.join("src").join("lib.rs"), lib).context("write lib.rs")?;
        }
    }

    let readme_t = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/README.generated.hbs"
    ));
    let readme = reg
        .render_template(readme_t, &ctx)
        .context("render README")?;
    fs::write(out.join("README.md"), readme).context("write README")?;

    if cfg.with_example_role {
        fs::create_dir_all(out.join("roles").join("default")).context("create roles/default")?;
        let st = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/settings.json.hbs"
        ));
        let settings = reg
            .render_template(st, &ctx)
            .context("render settings.json")?;
        fs::write(out.join("roles/default/settings.json"), settings)
            .context("write settings.json")?;
        fs::write(
            out.join("roles/default/character.md"),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/character.md"
            )),
        )
        .context("write character.md")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_keeps_alnum() {
        assert_eq!(slugify("  My_Project "), "my-project");
    }
}
