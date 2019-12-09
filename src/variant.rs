pub use crate::test::Test;
use serde::{Deserialize, Serialize};

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
