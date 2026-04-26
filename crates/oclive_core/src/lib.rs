//! # oclive_core — Oclive Kernel
//!
//! Platform-independent domain logic and OOCP protocol definitions.
//! This crate MUST NOT depend on `tauri`, OS windowing, or renderer crates.
//!
//! ## License
//! AGPL-3.0. See repository root LICENSE for the full license and
//! plugin exception terms.
//!
//! ## Structure (v0.1 initial skeleton)
//! - `oocp/` — OOCP v0.1 types (request/response/event envelopes)
//! - `capabilities/` — core capability declarations

pub mod capabilities;
pub mod oocp;