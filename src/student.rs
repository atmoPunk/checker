pub use crate::lab::LabError;
pub use crate::program::Program;
pub use crate::variant::Variant;
use async_std::fs;
use serde::{Deserialize, Serialize};
use std::time::Instant;

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
    pub async fn check(&self) -> Result<(), LabError> {
        if let Err(build_error) = self.program.build() {
            return Err(LabError::BuildError(build_error));
        }
        for i in 0..self.var.tests.len() {
            self.check_test(i).await? // '?' syntax - if we encounter a Err -> we return early and send it up
                                      // Else - we continure running
        }

        Ok(())
    }

    /// Checks a single test without building
    async fn check_test(&self, test_num: usize) -> Result<(), LabError> {
        let test = &self.var.tests[test_num];
        let start_time = Instant::now();
        let prog_output = self.program.run(&test.input);
        let duration = start_time.elapsed().as_secs_f64();
        if duration - test.time_limit > std::f64::EPSILON {
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
                    return Err(LabError::WrongAnswer(test_num));
                }
            }
            Err(e) => {
                return Err(LabError::RuntimeError(test_num, e)); // Program has encountered a runtime error
            }
        }

        Ok(())
    }
}
