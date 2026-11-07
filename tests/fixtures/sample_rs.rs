//! Root crate documentation for sample_rs fixture
//! Demonstrates complex Rust constructs, raw strings, and nested comments.

/// Telemetry metric record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRecord {
    pub id: u64,
    pub name: String,
    pub value: f64,
}

/* Outer block comment
   /* Nested level 1
      /* Nested level 2 comment */
   */
*/
impl MetricRecord {
    /// Creates a new metric record instance
    pub fn new(id: u64, name: &str, value: f64) -> Self {
        // Construct instance directly
        Self {
            id,
            name: name.to_string(),
            value,
        }
    }

    pub fn to_raw_json(&self) -> String {
        // Raw string literal containing comment symbols
        let raw = r#"{"comment": "// not removed", "block": "/* keep */"}"#;
        raw.to_string()
    }
}
