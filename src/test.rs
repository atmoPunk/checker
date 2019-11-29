use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    pub input: PathBuf,
    pub output: PathBuf,
    pub time_limit: f64,     // seconds
    pub memory_limit: usize, // bytes?
}

impl Test {
    pub fn new(input: PathBuf, output: PathBuf, time_limit: f64, memory_limit: usize) -> Self {
        Test {
            input,
            output,
            time_limit,
            memory_limit,
        }
    }
}
