use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub enum Lang {
    Cpp,
}

#[derive(Debug, Clone)]
pub struct Program {
    lang: Lang,
    path_to_src: PathBuf,
    path_to_exe: PathBuf,
}

#[derive(Debug, Clone)]
pub enum BuildError {
    ProcessSpawnError,
    CompilationError(String),
}

#[derive(Debug, Clone)]
pub enum RunError {
    RuntimeError(String),
    ProcessSpawnError,
}

impl Program {
    pub fn new(lang: Lang, path_to_src: PathBuf, path_to_exe: PathBuf) -> Self {
        Program {
            lang,
            path_to_src,
            path_to_exe,
        }
    }

    pub fn build(&self) -> Result<(), BuildError> {
        match self.lang {
            Lang::Cpp => {
                // Currently we are using user-made Makefiles
                let build = Command::new("make").current_dir(&self.path_to_src).output();
                match build {
                    Ok(output) => {
                        if !output.status.success() {
                            Err(BuildError::CompilationError(
                                String::from_utf8(output.stderr).unwrap(),
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    Err(_) => Err(BuildError::ProcessSpawnError),
                }
            }
        }
    }

    pub fn run(&self, input_file: &Path) -> Result<String, RunError> {
        let prog = Command::new(&self.path_to_exe)
            .stdin(File::open(input_file).expect("Can't open input_file"))
            .output();
        match prog {
            Ok(output) => {
                if !output.status.success() {
                    Err(RunError::RuntimeError(
                        String::from_utf8(output.stderr).unwrap(),
                    ))
                } else {
                    Ok(String::from_utf8(output.stdout).unwrap()) // Maybe send Vec<u8>, without converting?
                }
            }
            Err(_) => Err(RunError::ProcessSpawnError),
        }
    }
}
