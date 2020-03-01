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
    pub last_commit: Option<String>,
    pub repo: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerError {
    /// Corresponds to failure while launching build process or luanching program itself
    /// (Not a student's fault)
    ProcessSpawnError(String),
    /// Contains stderr of the build system
    CompilationError(String),
    /// Contains stderr of the failed program
    RuntimeError(String),
}

impl Program {
    pub fn new(lang: Lang, last_commit: Option<String>, repo: String, owner: String) -> Self {
        Program {
            lang,
            last_commit,
            repo,
            owner,
        }
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(&format!("students/{}/{}", &self.owner, &self.repo))
    }

    /// Tries to build a program
    pub fn build(&self) -> Result<(), RunnerError> {
        match self.lang {
            Lang::Cpp => {
                // Currently we are using user-made Makefiles
                let path_to_src = self.path();
                dbg!(&path_to_src);
                let build = Command::new("make").current_dir(path_to_src).output();
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
                    Err(e) => Err(RunnerError::ProcessSpawnError(e.to_string())),
                }
            }
        }
    }

    /// Runs a program, redirecting contents of input_file to stdin of program
    /// On success returns stdout of the program
    /// Panics if it can't open input file
    pub fn run(&self, input_file: &Path) -> Result<String, RunnerError> {
        let path = {
            let mut path = self.path();
            path.push("prog.out");
            path
        };
        let prog = Command::new(&path)
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
            Err(e) => Err(RunnerError::ProcessSpawnError(e.to_string())),
        }
    }
}
