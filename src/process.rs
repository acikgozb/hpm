//! A minimal module, acts as a wrapper around [`std::process::Command`].
//!
//! [`hpm::process`] is designed to provide a basic API on top of [`std::process::Command`]:
//!
//! - It provides a basic validation that checks whether the given program is accessible on the host (similar to `which` on Linux).
//! - It pipes the output streams [`std::io::stdout`] and [`std::io::stderr`] and delegates the (exit code, output stream) to clients accordingly.
//! - It's [`Error`] type makes the failure points of an execution easier to understand.
//!
//! [`hpm::process`]: crate::process
//! [`std::process::Command`]: std::process::Command
//! [`std::io::stdout`]: std::io::stdout
//! [`std::io::stderr`]: std::io::stderr
//! [`Error`]: crate::process::Error

use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

/// The main Error type of [`crate::process`].
///
/// It is designed to show each potential failure point during an execution of a [`std::process::Command`].
///
/// [`crate::process`]: crate::process
/// [`std::process::Command`]: std::process::Command
#[derive(Debug)]
pub enum Error {
    /// Represents a failed `$PATH` lookup of the program
    /// that is passed to [`crate::process::Process`].
    /// [`crate::process::Process`]: crate::process::Process
    BinaryDoesNotExist(OsString),

    /// Represents a failed execution of the given [`std::process::Command`]
    /// to [`crate::process::Process`].
    /// Provides the program name and the originated [`std::io::Error`].
    ///
    /// [`crate::process::Process`]: crate::process::Process
    /// [`std::io::Error`]: std::io::Error
    FailedToExecProcess(OsString, std::io::Error),

    /// Represents a successful execution of a [`std::process::Command`] that resulted in an error.
    /// Provides the exit code of the process, along with its [`std::io::stderr`] stream.
    ///
    /// [`crate::process::Process`]: crate::process::Process
    /// [`std::io::stderr`]: std::io::stderr
    Exec(u8, Vec<u8>),

    /// Represents an interruption during the execution of a given [`std::process::Command`].
    ///
    /// [`std::process::Command`]: std::process::Command
    Interrupted,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BinaryDoesNotExist(binary) => {
                write!(f, "the binary does not exist: {:?}", binary)
            }
            Error::FailedToExecProcess(binary, error) => {
                write!(f, "failed to execute the binary {:?}: {}", binary, error)
            }
            Error::Exec(_, vec) => {
                let process_err = String::from_utf8(vec.to_owned())
                    .expect("the process output should be valid UTF-8");

                write!(f, "{process_err}")
            }
            Error::Interrupted => {
                write!(f, "interrupted by the host")
            }
        }
    }
}

/// [`crate::process::Process`] is the main building block of [`crate::process`].
/// It wraps a user provided [`std::process::Command`] and provides a simple API to execute it safely.
///
/// Even though it can be instantiated manually by hand, prefer using the [`crate::process::Process::new`] method.
///
/// For more information, please refer to [`crate::process::Process::exec`].
///
/// [`crate::process`]: crate::process
/// [`crate::process::Process`]: crate::process::Process
/// [`crate::process::Process::new`]: crate::process::Process::new
/// [`crate::process::Process::exec`]: crate::process::Process::exec
/// [`std::process::Command`]: std::process::Command
pub struct Process(Command);

impl Process {
    /// Creates a new Process.
    pub fn new(cmd: Command) -> Self {
        Self(cmd)
    }

    fn get_process_name(&self) -> &OsStr {
        self.0.get_program()
    }

    fn validate(&self) -> Result<(), Error> {
        let process_name = self.get_process_name();
        which::which(process_name)
            .map(|_| ())
            .map_err(|_| Error::BinaryDoesNotExist(process_name.to_os_string()))
    }

    /// [`exec`] is the only meaningful interaction point of a [`crate::process::Process`].
    /// It validates the program of the user provided [`std::process::Command`],
    /// executes the command and waits it.
    ///
    /// If command result is Ok, then [`exec`] returns the [`std::io::stdout`] stream to the caller.
    /// If command result is Error, then [`exec`] returns the exit code along with the [`std::io::stderr`] stream of the command.
    ///
    /// # Errors
    ///
    /// [`crate::process::Error::FailedToExecProcess`] - Originates when the execution of Command fails.
    /// [`crate::process::Error::Interrupted`] - Originates when the execution of the command is interrupted.
    /// [`crate::process::Error::Exec`] - Originates when the Command is executed successfully, but the received exit code is greater than zero.
    /// It holds the exit code along with the [`std::io::stderr`] stream.
    ///
    /// [`exec`]: crate::process::Process::exec
    /// [`crate::process::Process`]: crate::process::Process
    /// [`crate::process::Error::Interrupted`]: crate::process::Error::Interrupted
    /// [`crate::process::Error::FailedToExecProcess`]: crate::process::Error::FailedToExecProcess
    /// [`crate::process::Error::Exec`]: crate::process::Error::Exec
    /// [`std::io::stdout`]: std::io::stdout
    /// [`std::io::stderr`]: std::io::stderr
    /// [`std::process::Process`]: std::process::Process
    pub fn exec(&mut self) -> Result<Vec<u8>, Error> {
        self.validate()?;

        let proc_output = self
            .0
            .output()
            .map_err(|err| Error::FailedToExecProcess(self.get_process_name().into(), err))?;

        let ecode = proc_output.status.code().ok_or(Error::Interrupted)? as u8;

        if proc_output.status.success() {
            return Ok(proc_output.stdout);
        }

        Err(Error::Exec(ecode, proc_output.stderr))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn should_return_proper_program_name() {
        let cmd = Command::new("echo");
        assert_eq!(Process::new(cmd).get_process_name(), "echo")
    }

    #[test]
    fn should_not_exec_nonexistent_binaries() {
        let cmd = Command::new("this-binary-does-not-exist");
        let mut process = Process::new(cmd);

        let validate_result = process.validate();
        assert!(validate_result.is_err());

        let exec_result = process.exec();
        assert!(exec_result.is_err());

        let validate_err_str = format!("{}", validate_result.unwrap_err());
        let exec_err_str = format!("{}", exec_result.unwrap_err());

        assert_eq!(validate_err_str, exec_err_str);
    }

    #[test]
    fn should_propagate_stderr_of_child_process() {
        let mut cmd = Command::new("ls");
        cmd.arg("this-file-does-not-exist");

        let mut process = Process::new(cmd);
        let exec_result = process.exec();

        assert!(exec_result.is_err_and(|err| {
            if let Error::Exec(ecode, stderr) = err {
                ecode > 0u8 && stderr.bytes().count() > 0
            } else {
                false
            }
        }));
    }

    #[test]
    fn should_propagate_stdout_of_child_process() {
        let cmd = Command::new("ls");

        let mut process = Process::new(cmd);
        let exec_result = process.exec();

        assert!(exec_result.is_ok_and(|stdout| { stdout.bytes().count() > 0 }));
    }
}
