pub use crate::github::{clone_repo, pull_repo};
pub use crate::lab::LabError;
pub use crate::program::Program;
pub use crate::variant::Variant;
pub use crate::github::RepoState;
use async_std::fs;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Instant;
use slog::{Logger, o, info};

/// Path to doxygen config
static DOXYFILE: &str = "/home/atmopunk/doxygen.config";

/// Holds path to program, current variant and last check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub program: Program,
    pub var: Variant,
    /// Possible variants: None -> No test were run ever
    /// Some(Ok(())) -> Student has passed all tests
    /// Some(Err(e)) -> Student has encountered some error
    pub result: Option<Result<(), LabError>>,
}

impl Student {
    pub fn new(program: Program, var: Variant, result: Option<Result<(), LabError>>) -> Self {
        Student {
            program,
            var,
            result,
        }
    }

    /** Builds a program written by student and then runs all tests from variant.
    Stops on the first test that fails.
    Panics if tests can't be opened or parsed to utf8 (we can read them as bytes) */
    pub async fn check(&self, logger: Logger) -> Result<(), LabError> {
        if let Err(build_error) = self.program.build() {
            info!(logger, "Build error");
            return Err(LabError::BuildError(build_error));
        }
        info!(logger, "Build success");
        for i in 0..self.var.tests.len() {
            let test_logger = logger.new(o!("test" => i));
            self.check_test(i, test_logger).await? // '?' syntax - if we encounter a Err -> we return early and send it up
                                      // else - we continure running
        }

        Ok(())
    }

    pub async fn download(&self) -> Result<RepoState, std::io::Error> {
        let cloned = clone_repo(&self.program.owner, &self.program.repo)?;
        if cloned == RepoState::Old {
            pull_repo(&self.program.owner, &self.program.repo)
        } else {
            Ok(cloned)
        }
    }

    pub fn build_doxygen(&self) -> Result<(), LabError> {
        let folder = self.program.path();
        println!("folder: {:?}", folder);
        let mut xmlfolder = folder.clone();
        xmlfolder.push("xml");
        println!("folder: {:?}", xmlfolder);

        let _doxygen = Command::new("doxygen")
            .arg(DOXYFILE)
            .current_dir(folder)
            .status()
            .unwrap();
        let xs = "xsltproc";
        let _concatenate = Command::new(xs)
            .current_dir(xmlfolder)
            .arg("combine.xslt")
            .arg("index.xml")
            .output()
            .unwrap();
        //concatenate.stdout TODO: dump this to all xml
        Ok(())
    }

    /// Checks a single test without building
    async fn check_test(&self, test_num: usize, logger: Logger) -> Result<(), LabError> {
        let test = &self.var.tests[test_num];
        let start_time = Instant::now();
        info!(logger, "started test");
        let prog_output = self.program.run(&test.input);
        info!(logger, "finished test");
        let duration = start_time.elapsed().as_secs_f64();
        if duration - test.time_limit > std::f64::EPSILON {
            info!(logger, "Result: timeout");
            return Err(LabError::TimeLimit(test_num));
        }
        match prog_output {
            Ok(output) => {
                let output = output.trim();
                let true_output = String::from_utf8(
                    fs::read(&test.output)
                        .await
                        .expect("Can't open test output"),
                )
                .expect("Can't parse test output");
                if output != true_output.trim() {
                    eprintln!("Got:\n{}", output); // TODO: send it up, so we can send it to students later
                    eprintln!("Expected:\n{}", true_output);
                    info!(logger, "Result: wrong answer");
                    return Err(LabError::WrongAnswer(test_num));
                }
            }
            Err(e) => {
                info!(logger, "Result: runtime error");
                return Err(LabError::RuntimeError(test_num, e)); // Program has encountered a runtime error
            }
        }
        info!(logger, "Result: ok");
        Ok(())
    }
}
