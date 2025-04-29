use clap::{Parser, Subcommand};
use hpm::Process;
use std::io::Write;
use std::process::ExitCode;

const PROGRAM: &str = "hpm";

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(hpm_err) => {
            eprintln!("{PROGRAM}: {hpm_err}");

            if let Some(err) = hpm_err.downcast_ref::<hpm::Error>() {
                return ExitCode::from(match err {
                    hpm::Error::BinaryDoesNotExist(_) => 1u8,
                    hpm::Error::FailedToExecProcess(_, _) => 1u8,
                    hpm::Error::Exec(ecode, _) => ecode.to_owned(),
                    hpm::Error::Interrupted => 130u8,
                });
            }

            if let Some(err) = hpm_err.downcast_ref::<Error>() {
                return ExitCode::from(match err {
                    Error::FailedToWriteStdout(_) => 1u8,
                });
            }

            ExitCode::from(2u8)
        }
    }
}

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
                write!(f, "failed to write to stdout: {}", err)
            }
        }
    }
}

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

    let user = std::env::var("USER").expect("$USER should be set for '{PROGRAM} logout'");
    cmd.arg(user);

    Process::new(cmd)
}
