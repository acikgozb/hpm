//! The API of `hpm`.

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

/// The main Error type of [`crate::cli`].
///
/// It is designed to capture [`std::io::Error`]s that originate from the underlying OS. It limits the scope to ones that originates during basic CLI actions (read, write).
///
/// [`crate::cli`]: crate::cli
/// [`std::io::Error`]: std::io::Error
#[derive(Debug)]
pub enum Error {
    /// Represents a failed write to [`std::io::stdout`].
    /// It holds the underlying [`std::io::Error`].
    ///
    /// [`std::io::stdout`]: std::io::stdout
    /// [`std::io::Error`]: std::io::Error
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

/// The main entrypoint of `hpm`.
///
/// `run` is designed to be used by CLI binary crates. It parses the shell arguments and then forwards the user to appropriate child processes.
///
/// If Result is Ok, it represents a successful `hpm` execution. This means that the child process is executed and its stdout stream is redirected to /dev/stdout.
///
/// If Result is Error, it is either a [`hpm::process::Error`] or [`hpm::cli::Error`].
/// `run` does not do anything internally when Result is Error, it is up to the caller to act accordingly (e.g. terminating the program with appropriate exit codes.)
///
/// [`hpm::process::Error`]: crate::process::Error
/// [`hpm::cli::Error`]: crate::cli::Error
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut process = if let Some(cmd) = args.command {
        match cmd {
            Command::Kill => kill(),
            Command::Restart => restart(),
            Command::Logout => logout(),
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

fn restart() -> Process {
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("reboot");

    Process::new(cmd)
}

fn logout() -> Process {
    let mut cmd = std::process::Command::new("loginctl");
    cmd.arg("terminate-user");

    let user =
        std::env::var("USER").expect("{PROGRAM}: $USER should be set for '{PROGRAM} restart'");
    cmd.arg(user);

    Process::new(cmd)
}
