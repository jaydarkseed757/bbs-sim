#![allow(dead_code)]

/// A parsed segment of text with optional color attributes.
pub struct AnsiSegment {
    pub text: String,
    pub fg: Option<u8>,
    pub bg: Option<u8>,
    pub bold: bool,
}

/// Stub ANSI escape-code parser — to be implemented in a later build step.
pub struct AnsiParser;

impl AnsiParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Vec<AnsiSegment> {
        vec![AnsiSegment {
            text: input.to_string(),
            fg: None,
            bg: None,
            bold: false,
        }]
    }
}
