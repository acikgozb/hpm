use clap::{Parser, Subcommand};
use hpm::Process;
use std::collections::HashMap;
use std::fmt::{self, Debug};
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
                    Error::FailedToReadStdin(_) => 1u8,
                    Error::InvalidUserAnswer => 1u8,
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

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Kill => write!(f, "Kill"),
            Command::Restart => write!(f, "Restart"),
            Command::Logout => write!(f, "Logout"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    FailedToWriteStdout(std::io::Error),
    FailedToReadStdin(std::io::Error),
    InvalidUserAnswer,
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FailedToWriteStdout(err) => {
                write!(f, "failed to write to stdout: {}", err)
            }
            Error::FailedToReadStdin(err) => {
                write!(f, "failed to read stdin: {}", err)
            }
            Error::InvalidUserAnswer => {
                write!(f, "the given command does not exist")
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cmd = if args.interactive {
        interactive()?
    } else if let Some(cmd) = args.command {
        cmd
    } else {
        return Ok(());
    };

    let mut process = match cmd {
        Command::Kill => kill(),
        Command::Restart => restart(),
        Command::Logout => logout(),
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

fn interactive() -> Result<Command, Error> {
    let cmds = [Command::Kill, Command::Restart, Command::Logout];

    let mut prompt_str = String::new();
    let mut cmd_map: HashMap<u8, Command> = HashMap::new();

    for (idx, cmd) in cmds.into_iter().enumerate() {
        prompt_str = format!("{}, ({}) {}", prompt_str, idx, cmd);
        cmd_map.insert(idx as u8, cmd);
    }

    let prompt_str = prompt_str.strip_prefix(", ").unwrap();
    let mut answer_buf = String::new();

    println!("Select the command you wish to execute:\n{}", prompt_str);
    std::io::stdin()
        .read_line(&mut answer_buf)
        .map_err(Error::FailedToReadStdin)?;

    let cmd_key = answer_buf
        .trim()
        .parse::<u8>()
        .map_err(|_| Error::InvalidUserAnswer)?;
    let selected_cmd = cmd_map.remove(&cmd_key).ok_or(Error::InvalidUserAnswer)?;

    Ok(selected_cmd)
}
