/*!
 * Validators module
 *
 * Common validation types and utilities
 */

pub struct ValidationResult {
    pub stage_name: String,
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct ValidationError {
    pub line: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct ValidationWarning {
    pub line: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

impl ValidationResult {
    pub fn new(stage_name: &str) -> Self {
        Self {
            stage_name: stage_name.to_string(),
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn error(&mut self, line: Option<usize>, message: String, suggestion: Option<String>) {
        self.passed = false;
        self.errors.push(ValidationError {
            line,
            message,
            suggestion,
        });
    }

    pub fn warning(&mut self, line: Option<usize>, message: String, suggestion: Option<String>) {
        self.warnings.push(ValidationWarning {
            line,
            message,
            suggestion,
        });
    }

    pub fn passed(&self) -> bool {
        self.passed
    }

    pub fn is_passing(&self) -> bool {
        self.passed
    }

    pub fn add_metadata(&mut self, key: &str, value: String) {
        self.metadata.insert(key.to_string(), value);
    }
}
