use serde_json::Value;

/// An individual test case, containing multiple tests of a single schema's behavior.
#[derive(Debug, serde::Deserialize)]
pub struct Case {
    /// The test case description.
    pub description: String,
    /// A valid JSON Schema.
    pub schema: Value,
    /// A set of related tests all using the same schema.
    pub tests: Vec<InnerTest>,
}

/// A single test.
#[derive(Debug, serde::Deserialize)]
pub struct InnerTest {
    /// The test description, briefly explaining which behavior it exercises.
    pub description: String,
    /// Any additional comments about the test.
    pub comment: Option<String>,
    /// The instance which should be validated against the schema in schema.
    pub data: Value,
    /// Whether the validation process of this instance should consider the instance valid or not.
    pub valid: bool,
}

#[derive(Debug)]
pub struct Test {
    pub draft: &'static str,
    pub schema: Value,
    pub case: &'static str,
    pub is_optional: bool,
    /// The test description, briefly explaining which behavior it exercises.
    pub description: &'static str,
    /// The instance which should be validated against the schema in schema.
    pub data: Value,
    /// Whether the validation process of this instance should consider the instance valid or not.
    pub valid: bool,
}
