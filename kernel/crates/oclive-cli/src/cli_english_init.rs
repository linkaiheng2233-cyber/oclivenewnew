//! English `init --help` footer (preset matrix).

pub const PRESET_MATRIX_HELP: &str = r#"Preset and plugin_backends (logical slots)

┌───────────────────┬─────────┬─────────┬────────┐
│ Slot              │ minimal │ mixed   │ full   │
├───────────────────┼─────────┼─────────┼────────┤
│ memory            │ builtin │ builtin │ builtin│
│ emotion           │ builtin │ builtin │ builtin│
│ event             │ builtin │ builtin │ builtin│
│ prompt            │ builtin │ builtin │ builtin│
│ llm               │ ollama  │ ollama  │ remote │
│ agent             │ none*   │ builtin │ builtin│
│ complex_emotion   │ none    │ builtin │ remote │
└───────────────────┴─────────┴─────────┴────────┘

* agent = none: omit the agent key in settings.json (host falls back to builtin).

llm = ollama uses the in-process local client; use remote + OCLIVE_REMOTE_LLM_URL if no local model (see PLUGIN_V1).

Monolith (kernel_server only): add --monolith in non-interactive mode. oclive build reads monolith.toml. See RFC_OCLIVE_MONOLITH_MODE.md.

Factory templates (--template; explicit CLI flags override defaults):

┌─────────────────┬─────────┬──────────────────┬────────────────┬──────────────────────────────┐
│ template        │ preset  │ monolith default │ project-type   │ default --with-role-pack     │
├─────────────────┼─────────┼──────────────────┼────────────────┼──────────────────────────────┤
│ robot-soul      │ minimal │ on               │ kernel_server  │ robot-soul-minimal           │
│ robot-gateway   │ mixed   │ on               │ kernel_server  │ gateway + mcp_servers/       │
│ dialogue-only   │ full    │ off              │ kernel_server  │ default                      │
│ headless-api    │ full    │ off              │ kernel_server  │ none (empty roles/)          │
│ library-embed   │ minimal │ off              │ library        │ none                         │
└─────────────────┴─────────┴──────────────────┴────────────────┴──────────────────────────────┘

--monolith-preset: latency (all slots) | memory | embedded.

--with-role-pack: robot-soul-minimal | default. --skip-role-pack forces empty roles/.

--with-example-plugin: copies examples/directory-plugin-llamacpp/ (off by default).

--list-templates: print matrix and exit.

--monolith-bench-preset: post-init release bench (5 runs) to bench_results/report.json (non-blocking).

--quick / -q: full preset, no Monolith, no roles, no --kernel-source; interactive asks name + output only.

--dual-core: example role pack uses pipeline.ocblueprint schema_version 3 with runtime_config.dual_core.enabled (requires roles/; pair with --monolith for monolith.toml [dual_core]).
"#;
