pub use crate::runner::{Program, RunnerError};
pub use crate::test::Test;
use async_std::fs;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Holds current students in a HashMap (Student name -> Student struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lab {
    pub students: HashMap<String, Student>,
}

impl Lab {
    pub fn new(students: HashMap<String, Student>) -> Self {
        Lab { students }
    }

    /// Starts checking current lab for this student
    pub async fn check(&self, student: &str) -> Result<(), LabError> {
        self.students[student].check().await
    }

    /// Checks all students
    pub async fn check_all(&self) -> HashMap<String, Result<(), LabError>> {
        let mut checks = Vec::new();
        let mut names = Vec::new();
        for (name, s) in self.students.iter() {
            names.push(name);
            checks.push(s.check());
        }
        let checks_finish = join_all(checks).await;
        let mut result = HashMap::new();
        for i in 0..names.len() {
            result.insert(names[i].to_owned(), checks_finish[i].to_owned());
        }
        result
    }
}

/// Holds paths to test files for the variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub tests: Vec<Test>,
}

impl Variant {
    pub fn new(tests: Vec<Test>) -> Self {
        Variant { tests }
    }
}

/// Holds path to program, current variant and last check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub program: Program,
    pub var: Variant,
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

    /** Builds a program written by student and then runs all tests from variant sequentually.
    Stops on the first test that fails.
    Panics if tests can't be opened or parsed to utf8 (we can read them as bytes) */
    pub async fn check(&self) -> Result<(), LabError> {
        if let Err(build_error) = self.program.build() {
            return Err(LabError::BuildError(build_error));
        }
        for (i, test) in self.var.tests.iter().enumerate() {
            let start = Instant::now();
            let out = self.program.run(&test.input); // Running program
            let dur = start.elapsed().as_secs_f64();
            if dur - test.time_limit > std::f64::EPSILON {
                return Err(LabError::TimeLimit(i));
            }
            match out {
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
                        return Err(LabError::WrongAnswer(i));
                    }
                }
                Err(e) => {
                    return Err(LabError::RuntimeError(i, e)); // Program has encountered a runtime error
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabError {
    /// Contains # of test where program failed
    WrongAnswer(usize),
    /// Contains # of test where program failed
    TimeLimit(usize),
    /// Contains # of test where program failed and error message
    RuntimeError(usize, RunnerError),
    /// Contains error message
    BuildError(RunnerError),
}
