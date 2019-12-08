use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Abstraction of single test in a lab
/// Has path to file with input, with expected output,
/// Time limit in seconds, and memory limit (currently not implemented)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    pub input: PathBuf,
    pub output: PathBuf,
    /// Maximum number of seconds to execute a program
    pub time_limit: f64,
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
