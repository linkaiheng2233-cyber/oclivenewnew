//! Cross-platform setup for child processes that belong to the GUI host.

use std::process::Command;

/// Keep managed child processes detached from a visible Windows console.
pub(crate) fn configure_background_process(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(windows))]
    let _ = command;
}
