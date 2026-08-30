//! Desktop kernel lifecycle: discover → attach or spawn → supervise.

mod connection;
mod ensure;
mod policy;
mod port_ops;
pub mod reconnect;
pub mod spawn;
pub mod status;
mod watchdog;

pub use connection::{
    DesktopKernelMode, KernelConnection, KernelConnectionStatus, SharedKernelConnection,
};
pub use ensure::{ensure_kernel_ready, ensure_kernel_ready_on_conn, EnsureKernelOptions};
pub use reconnect::{
    emit_kernel_status, reconnect_once, AutoReconnectPolicy, ReconnectOptions, StatusEmit,
};
pub use status::{probe_health_status, to_ui_mode};
pub use watchdog::start_kernel_watchdog;
