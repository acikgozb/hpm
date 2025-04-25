use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    /// Select the subcommand interactively.
    #[arg(short, long, group = "standalone")]
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
pub enum HpmError {}

impl std::fmt::Display for HpmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "will be implemented.")
    }
}

pub fn run() -> Result<(), HpmError> {
    let cli = Cli::parse();

    println!("{:?}", cli);
    Ok(())
}
