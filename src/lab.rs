pub use crate::program::{Program, RunnerError};
pub use crate::student::Student;
pub use crate::test::Test;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use slog::{Drain, Logger, o};

fn create_default_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!("version" => "0.1"))
}

/// Holds current students in a HashMap (Student name -> Student struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lab {
    pub students: HashMap<String, Student>,

    #[serde(skip, default="create_default_logger")]
    logger: Logger,
}

impl Lab {
    pub fn new(students: HashMap<String, Student>) -> Self {
        Lab { students, logger: create_default_logger() }
    }

    /// Starts checking current lab for this student
    pub async fn check(&self, student: &str) -> Result<(), LabError> {
        let student_logger = self.logger.new(o!("student" => student.to_owned()));
        self.students[student].check(student_logger).await
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
            let student_logger = self.logger.new(o!("student" => name.to_owned()));
            checks.push(s.check(student_logger));
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
