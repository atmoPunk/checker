use std::path::PathBuf;
use std::process::Command;

pub enum Lang {
    Cpp,
}

pub struct Program {
    lang: Lang,
    path_to_src: PathBuf,
    path_to_exe: PathBuf,
}

pub enum BuildError {
    ProccesSpawnError,
    CompilationError(String),
}

pub enum RunError {
}

impl Program {
    pub fn build(&self) -> Result<(), BuildError> {
        match self.lang {
            Lang::Cpp => {
                // TODO: Currently we are using user-made Makefiles
                let build = Command::new("make").arg(self.path_to_src.clone()).output();
                match build {
                    Ok(output) => {
                        if !output.status.success() {
                            Err(BuildError::CompilationError(String::from_utf8(output.stdout).unwrap()))
                        } else {
                            Ok(())
                        }
                    },
                    Err(_) => Err(BuildError::ProccesSpawnError)
                }
            }
        }
    }
    
    pub fn run(&self, input_file: String, output_file: String) -> Result<(), RunError> {
        let prog = Command::new(self.path_to_exe.clone());
        unimplemented!()
    }
}
