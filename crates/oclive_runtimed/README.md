# oclive_runtimed (experimental, not in workspace)

Per-role HTTP request queue + kernel health supervisor prototype (Phase 3).

This crate is **not** wired into the desktop host, CLI, CI, or distro lifecycle. It remains in-tree for reference only.

To build locally:

```bash
cargo build --manifest-path crates/oclive_runtimed/Cargo.toml
```

**Environment:** `OCLIVE_KERNEL_UPSTREAM` (default `http://127.0.0.1:8420`) points at the real kernel; `OCLIVE_SCHEDULER_PORT` (default **8430**) is this proxy's listen port. Do not bind the scheduler on **8420** while a kernel is already listening there—use the default or a distinct port.
