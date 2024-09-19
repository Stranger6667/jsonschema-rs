use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
pub struct Case {
    /// A collection of schemas, identified by (retrieval) URI which may be referenced in tests in this file.
    #[serde(default)]
    pub registry: HashMap<String, serde_json::Value>,
    pub tests: Vec<InnerTest>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InnerTest {
    pub base_uri: Option<String>,
    #[serde(rename = "ref")]
    pub reference: String,
    pub target: Option<serde_json::Value>,
    /// A further test to run which should maintain state from the initial lookup.
    pub then: Option<Box<InnerTest>>,
    pub error: Option<bool>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Test {
    #[serde(default)]
    pub registry: HashMap<String, serde_json::Value>,
    pub base_uri: Option<&'static str>,
    #[serde(rename = "ref")]
    pub reference: &'static str,
    pub target: Option<serde_json::Value>,
    /// A further test to run which should maintain state from the initial lookup.
    pub then: Option<Box<Test>>,
    pub error: Option<bool>,
}
