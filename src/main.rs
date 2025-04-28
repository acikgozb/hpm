use std::process::ExitCode;

use hpm::{cli, process};

fn main() -> ExitCode {
    match hpm::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(hpm_err) => {
            eprintln!("{hpm_err}");

            if let Some(err) = hpm_err.downcast_ref::<process::Error>() {
                return ExitCode::from(match err {
                    process::Error::BinaryDoesNotExist(_) => 1u8,
                    process::Error::FailedToExecProcess(_, _) => 1u8,
                    process::Error::Exec(ecode, _) => ecode.to_owned(),
                    process::Error::Interrupted => 130u8,
                });
            }

            if let Some(err) = hpm_err.downcast_ref::<cli::Error>() {
                return ExitCode::from(match err {
                    cli::Error::FailedToWriteStdout(_) => 1u8,
                });
            }

            ExitCode::from(2u8)
        }
    }
}
