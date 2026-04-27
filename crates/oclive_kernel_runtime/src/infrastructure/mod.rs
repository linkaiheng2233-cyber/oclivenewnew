// Temporary shim: selectively re-export infrastructure modules from `oclivenewnew-tauri`.
//
// Goal: progressively migrate code here and remove this dependency.

pub mod db;
pub mod repositories_runtime;

pub use oclivenewnew_tauri::infrastructure::{
    directory_plugins, llm, ollama_client, storage,
};
