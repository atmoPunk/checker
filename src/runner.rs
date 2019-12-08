use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Lang {
    Cpp,
}

/// Abstraction around students program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    lang: Lang,
    /// Path to Makefile
    path_to_src: PathBuf,
    /// Path to executable after it's built
    path_to_exe: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerError {
    ProcessSpawnError,
    /// Contains stderr of the build system
    CompilationError(String),
    /// Contains stderr of the failed program
    RuntimeError(String),
}

impl Program {
    pub fn new(lang: Lang, path_to_src: PathBuf, path_to_exe: PathBuf) -> Self {
        Program {
            lang,
            path_to_src,
            path_to_exe,
        }
    }

    /// Tries to build a program
    pub fn build(&self) -> Result<(), RunnerError> {
        match self.lang {
            Lang::Cpp => {
                // Currently we are using user-made Makefiles
                let build = Command::new("make").current_dir(&self.path_to_src).output();
                match build {
                    Ok(output) => {
                        if !output.status.success() {
                            Err(RunnerError::CompilationError(
                                String::from_utf8(output.stderr).unwrap(),
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    Err(_) => Err(RunnerError::ProcessSpawnError),
                }
            }
        }
    }

    /// Runs a program, redirecting contents of input_file to stdin of program
    /// On success returns stdout of the program
    pub fn run(&self, input_file: &Path) -> Result<String, RunnerError> {
        let prog = Command::new(&self.path_to_exe)
            .stdin(File::open(input_file).expect("Can't open input_file"))
            .output();
        match prog {
            Ok(output) => {
                if !output.status.success() {
                    Err(RunnerError::RuntimeError(
                        String::from_utf8(output.stderr).unwrap(),
                    ))
                } else {
                    Ok(String::from_utf8(output.stdout).unwrap()) // Maybe send Vec<u8>, without converting?
                }
            }
            Err(_) => Err(RunnerError::ProcessSpawnError),
        }
    }
}
