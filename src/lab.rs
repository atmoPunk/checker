use crate::runner::Program;
use crate::test::Test;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Lab {
    pub students: HashMap<String, Student>,
}

impl Lab {
    pub fn new(students: HashMap<String, Student>) -> Self {
        Lab { students }
    }
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub tests: Vec<Test>,
}

impl Variant {
    pub fn new(tests: Vec<Test>) -> Self {
        Variant { tests }
    }
}

#[derive(Debug, Clone)]
pub struct LabConfig {
    pub var_num: usize,
    pub vars: Vec<Variant>,
}

#[derive(Debug, Clone)]
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

    pub fn check(&self) -> Result<(), LabError> {
        self.program.build().expect("Can't build a program");
        for (i, test) in self.var.tests.iter().enumerate() {
            match self.program.run(&test.input) {
                Ok(output) => {
                    let true_output =
                        String::from_utf8(fs::read(&test.output).expect("Can't open test output"))
                            .expect("Can't parse test output");
                    if output.trim() != true_output.trim() {
                        eprintln!("Got:\n{}", output);
                        eprintln!("Expected:\n{}", true_output);
                        return Err(LabError::WrongAnswer(i));
                    }
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    return Err(LabError::Error(i));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum LabError {
    WrongAnswer(usize),
    Error(usize),
}
