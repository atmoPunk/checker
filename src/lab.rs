pub use crate::program::{Program, RunnerError};
pub use crate::student::Student;
pub use crate::test::Test;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn build_doxygen(&self, student: &str) -> Result<(), LabError> {
        self.students[student].build_doxygen()
    }

    /// Checks all students
    pub async fn check_all(&self) -> HashMap<String, Result<(), LabError>> {
        let mut checks = Vec::with_capacity(self.students.len());
        let mut names = Vec::with_capacity(self.students.len());
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

    pub async fn download_all(&self) -> Result<(), std::io::Error> {
        let mut downloads = Vec::with_capacity(self.students.len());
        for (_, s) in self.students.iter() {
            downloads.push(s.download());
        }
        let downloads: Result<Vec<_>, _> = join_all(downloads).await.into_iter().collect();
        downloads?;
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
