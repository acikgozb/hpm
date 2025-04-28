use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

use crate::PROGRAM;

#[derive(Debug)]
pub enum Error {
    BinaryDoesNotExist(OsString),
    FailedToExecProcess(OsString, std::io::Error),
    Exec(u8, Vec<u8>),
    Interrupted,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BinaryDoesNotExist(binary) => {
                write!(f, "{PROGRAM}: the binary does not exist: {:?}", binary)
            }
            Error::FailedToExecProcess(binary, error) => {
                write!(
                    f,
                    "{PROGRAM}: failed to execute the binary {:?}: {}",
                    binary, error
                )
            }
            Error::Exec(_, vec) => {
                let process_err = String::from_utf8(vec.to_owned())
                    .expect("the process output should be valid UTF-8");

                write!(f, "{PROGRAM}: {process_err}")
            }
            Error::Interrupted => {
                write!(f, "{PROGRAM}: interrupted by the host")
            }
        }
    }
}

pub struct Process(Command);
impl Process {
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
