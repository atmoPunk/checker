pub use crate::github::RepoState;
pub use crate::program::{Program, RunnerError};
pub use crate::student::Student;
pub use crate::test::Test;
pub use crate::ast::{gen_hashes, compare};

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use slog::{o, Drain, Logger};
use std::collections::HashMap;
use lettre::{Transport, SmtpTransport};

fn create_default_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build(); // TODO: log to file
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!("version" => "0.1"))
}

pub fn notify(name1: String, name2: String, coef: f32) {
    let email = lettre_email::EmailBuilder::new()
        .to("teacher@mail.ru") // TODO
        .from("teacher@mail.ru")
        .subject("Hello")
        .text(&(name1 + &String::from(" ") + &name2 + &String::from(": ") + &coef.to_string()))
        .build()
        .unwrap();

    let tls_builder = native_tls::TlsConnector::builder();

    let tls_parameters = lettre::ClientTlsParameters::new("smtp.yandex.ru".to_string(), tls_builder.build().unwrap());

    let mut mailer = lettre::SmtpClient::new(
        ("smtp.yandex.ru", 465),
        lettre::ClientSecurity::Wrapper(tls_parameters)).unwrap()
            .authentication_mechanism(lettre::smtp::authentication::Mechanism::Login)
            .credentials(lettre::smtp::authentication::Credentials::new(String::from("teacher@mail.ru"), String::from("password")))
        .transport();

    mailer.send(email.into()).unwrap();
}

/// Holds current students in a HashMap (Student name -> Student struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lab {
    pub students: HashMap<String, Student>,

    #[serde(skip, default = "create_default_logger")]
    logger: Logger,
}

impl Lab {
    pub fn new(students: HashMap<String, Student>) -> Self {
        Lab {
            students,
            logger: create_default_logger(),
        }
    }

    /// Starts checking current lab for this student
    pub async fn check(&self, student: &str) -> Result<(), LabError> {
        let student_logger = self.logger.new(o!("student" => student.to_owned()));
        self.students[student].check(student_logger).await
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

    pub async fn check_students(
        &self,
        names: Vec<String>,
    ) -> HashMap<String, Result<(), LabError>> {
        let mut checks = Vec::with_capacity(names.len());
        for (name, s) in self
            .students
            .iter()
            .filter(|&(name, _)| names.iter().any(|n| name == n))
        {
            let student_logger = self.logger.new(o!("student" => name.to_owned()));
            checks.push(s.check(student_logger));
        }
        let checks_finish = join_all(checks).await;
        let mut result = HashMap::new();
        for i in 0..names.len() {
            result.insert(names[i].to_owned(), checks_finish[i].to_owned());
            if checks_finish[i].is_ok() {
                gen_hashes(self.students[&names[i]].program.path(), self.students[&names[i]].program.path().to_str().unwrap().to_owned() + "Hashes");
            }
            for j in 0..i {
                let v = compare(self.students[&names[j]].program.path().to_str().unwrap().to_owned() + "Hashes", self.students[&names[i]].program.path().to_str().unwrap().to_owned() + "Hashes").unwrap();
                if v > 0.4 {
                    notify(names[i].clone(), names[j].clone(), v);
                }
            }
        }
        result
    }

    /// Returns names of students, which repo state changed
    pub async fn download_all(&self) -> Result<Vec<String>, std::io::Error> {
        let mut downloads = Vec::with_capacity(self.students.len());
        let mut students = Vec::with_capacity(self.students.len());
        for (name, s) in self.students.iter() {
            let student_logger = self.logger.new(o!("student" => name.to_owned()));
            students.push(name.clone());
            downloads.push(s.download(student_logger));
        }
        let downloads: Result<Vec<RepoState>, _> = join_all(downloads).await.into_iter().collect();
        let downloads = downloads?;
        let students: Vec<String> = students
            .into_iter()
            .enumerate()
            .filter(|&(i, _)| downloads[i] == RepoState::Updated)
            .map(|(_, name)| name)
            .collect();
        Ok(students)
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
