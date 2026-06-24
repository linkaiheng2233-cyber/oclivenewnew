//! Shared path helpers for integration tests (monorepo layout SSOT).

#![allow(dead_code)]

use oclive_kernel_runtime::{chat_pro_roles_dir, find_monorepo_root};
use std::path::PathBuf;

/// Monorepo root (`oclivenewnew`) containing `distros/desktop-tauri` and `distros/chat-pro/roles`.
pub fn monorepo_root() -> PathBuf {
    find_monorepo_root(&[PathBuf::from(env!("CARGO_MANIFEST_DIR"))])
        .expect("monorepo root with distros/chat-pro/roles")
}

/// Canonical Chat Pro role packs directory (`distros/chat-pro/roles`).
pub fn roles_dir() -> PathBuf {
    chat_pro_roles_dir(&[PathBuf::from(env!("CARGO_MANIFEST_DIR"))])
        .expect("distros/chat-pro/roles")
}
