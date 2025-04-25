use std::process::ExitCode;

mod cli;

fn main() -> ExitCode {
    match cli::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(hpm_err) => {
            eprintln!("{hpm_err}");
            ExitCode::FAILURE
        }
    }
}
