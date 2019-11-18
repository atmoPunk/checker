use std::path::PathBuf;
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
    ProccesSpawnError,
    CompilationError(String),
}

#[derive(Debug, Clone)]
pub enum RunError {}

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
                // TODO: Currently we are using user-made Makefiles
                let build = Command::new("make")
                    .current_dir(self.path_to_src.clone())
                    .output();
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
                    Err(_) => Err(BuildError::ProccesSpawnError),
                }
            }
        }
    }

    pub fn run(&self, input_file: String, output_file: String) -> Result<(), RunError> {
        let prog = Command::new(self.path_to_exe.clone());
        unimplemented!()
    }
}
