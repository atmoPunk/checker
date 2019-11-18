use crate::runner::Program;
use crate::test::{Test, TestResult};
use std::collections::HashMap;

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
    pub result: Option<TestResult>,
}

impl Student {
    pub fn new(program: Program, var: Variant, result: Option<TestResult>) -> Self {
        Student {
            program,
            var,
            result,
        }
    }
}
