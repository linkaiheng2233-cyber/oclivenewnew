//! `oclive completions <shell>` — shell completion scripts.

use anyhow::Result;
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, shells};
use std::io::stdout;

#[derive(Parser, Debug)]
pub struct CompletionsArgs {
    pub shell: ShellKind,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    #[value(alias = "powershell")]
    PowerShell,
}

pub fn run(args: CompletionsArgs) -> Result<()> {
    let mut cmd = crate::Cli::command();
    let name = "oclive";
    match args.shell {
        ShellKind::Bash => generate(shells::Bash, &mut cmd, name, &mut stdout()),
        ShellKind::Zsh => generate(shells::Zsh, &mut cmd, name, &mut stdout()),
        ShellKind::Fish => generate(shells::Fish, &mut cmd, name, &mut stdout()),
        ShellKind::PowerShell => generate(shells::PowerShell, &mut cmd, name, &mut stdout()),
    }
    Ok(())
}
