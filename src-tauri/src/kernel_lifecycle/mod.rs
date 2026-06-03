//! Desktop kernel lifecycle: discover → attach or spawn → supervise.

mod connection;
mod ensure;
pub mod spawn;
mod watchdog;

pub use connection::{DesktopKernelMode, KernelConnection, KernelConnectionStatus, SharedKernelConnection};
pub use ensure::{ensure_kernel_ready, EnsureKernelOptions};
pub use watchdog::start_kernel_watchdog;
