pub use crate::program::{Program, RunnerError};
pub use crate::student::Student;
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
