use std::io::Write;

use clap::{Parser, Subcommand};

use crate::{PROGRAM, process::Process};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Args {
    /// Open interactive mode.
    #[arg(short, long)]
    interactive: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Power off the system.
    Kill,

    /// Restart the system.
    Restart,

    /// Logout from the current $USER.
    Logout,
}

#[derive(Debug)]
pub enum Error {
    FailedToWriteStdout(std::io::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FailedToWriteStdout(err) => {
                write!(f, "{}: failed to write to stdout: {}", PROGRAM, err)
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut process = if let Some(cmd) = args.command {
        match cmd {
            Command::Kill => kill(),
            Command::Restart => todo!(),
            Command::Logout => todo!(),
        }
    } else {
        return Ok(());
    };

    let process_stdout = process.exec()?;

    std::io::stdout()
        .write_all(&process_stdout)
        .map_err(Error::FailedToWriteStdout)?;

    Ok(())
}

fn kill() -> Process {
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("poweroff");
    Process::new(cmd)
}
