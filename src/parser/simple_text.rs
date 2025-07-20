use crate::outline::Outline;
use anyhow::Result;
use regex::Regex;

/// Options for configuring the `SimpleTextParser`.
pub struct SimpleTextParserOptions {
    /// The string used for indentation (e.g., "  " for two spaces, "\t" for tab).
    pub indent: String,
    /// An optional delimiter string used to separate the key from its values.
    pub delimiter: Option<String>,
    /// If true, empty lines in the input will be preserved as level-1 items.
    pub preserve_empty_line: bool,
    /// A list of strings representing the key headers.
    pub key_header: Vec<String>,
    /// A list of strings representing the value headers.
    pub value_header: Vec<String>,
}

impl Default for SimpleTextParserOptions {
    /// Returns the default options for `SimpleTextParser`.
    ///
    /// Default values:
    /// - `indent`: "\t" (tab)
    /// - `delimiter`: None
    /// - `preserve_empty_line`: false
    /// - `key_header`: empty vector
    /// - `value_header`: empty vector
    fn default() -> Self {
        SimpleTextParserOptions {
            indent: "\t".to_string(),
            delimiter: None,
            preserve_empty_line: false,
            key_header: Vec::new(),
            value_header: Vec::new(),
        }
    }
}

/// A parser for converting simple text format into an `Outline` structure.
pub struct SimpleTextParser {
    option: SimpleTextParserOptions,
}

impl SimpleTextParser {
    /// Creates a new `SimpleTextParser` with the given options.
    ///
    /// # Arguments
    ///
    /// * `option` - The `SimpleTextParserOptions` to configure the parser.
    pub fn new(option: SimpleTextParserOptions) -> Self {
        SimpleTextParser {
            option,
        }
    }

    /// Parses the input string and converts it into an `Outline` structure.
    ///
    /// # Arguments
    ///
    /// * `input` - The string containing the simple text to parse.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Outline)` on successful parsing, or an `anyhow::Error`
    /// if an error occurs (e.g., invalid regex).
    pub fn parse(&self, input: &str) -> Result<Outline> {
        let indent_regexp = Regex::new(&format!(
            "^(?P<indents>({})+)",
            regex::escape(&self.option.indent)
        ))?;
        let delimiter_regexp = if let Some(d) = &self.option.delimiter {
            Some(Regex::new(&regex::escape(d))?)
        } else {
            None
        };

        let mut outline = Outline::new();
        outline.key_header = self.option.key_header.clone();
        outline.value_header = self.option.value_header.clone();

        for line in input.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() && !self.option.preserve_empty_line {
                continue;
            }

            let mut level = 1;
            let mut current_line = line.to_string();

            if !self.option.indent.is_empty() {
                if let Some(captures) = indent_regexp.captures(&current_line) {
                    let indents = captures.name("indents").unwrap().as_str();
                    level = 1 + (indents.len() / self.option.indent.len()) as u32;
                    current_line = indent_regexp.replace(&current_line, "").to_string();
                }
            }

            let key;
            let value;

            if let Some(d_regexp) = &delimiter_regexp {
                let parts: Vec<&str> = d_regexp.split(&current_line).collect();
                key = parts[0].trim().to_string();
                value = parts[1..].iter().map(|s| s.trim().to_string()).collect();
            } else {
                key = current_line.trim().to_string();
                value = Vec::new();
            }

            outline.add_item(&key, level, value);
        }

        Ok(outline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a reference Outline for testing.
    fn reference_outline() -> Outline {
        let mut outline = Outline::new();
        outline.key_header = vec!["H1".to_string(), "H2".to_string(), "H3".to_string()];
        outline.value_header = vec!["H(1)".to_string(), "H(2)".to_string()];
        outline.add_item("1", 1, vec!["1(1)".to_string(), "1(2)".to_string()]);
        outline.add_item("1.1", 2, vec!["1.1(1)".to_string(), "1.1(2)".to_string()]);
        outline.add_item("1.2", 2, vec!["1.2(1)".to_string(), "1.2(2)".to_string()]);
        outline.add_item("1.2.1", 3, vec!["1.2.1(1)".to_string(), "1.2.1(2)".to_string()]);
        outline
    }

    #[test]
    fn test_simple_text_parser_options_default() {
        let options = SimpleTextParserOptions::default();
        assert_eq!(options.indent, "\t");
        assert_eq!(options.delimiter, None);
        assert_eq!(options.preserve_empty_line, false);
        assert!(options.key_header.is_empty());
        assert!(options.value_header.is_empty());
    }

    #[test]
    fn test_simple_text_parser_new() {
        let options = SimpleTextParserOptions {
            indent: "  ".to_string(),
            delimiter: Some("\t".to_string()),
            value_header: vec!["H(1)".to_string(), "H(2)".to_string()],
            preserve_empty_line: true,
            ..Default::default()
        };
        let parser = SimpleTextParser::new(options);
        assert_eq!(parser.option.indent, "  ");
        assert_eq!(parser.option.delimiter, Some("\t".to_string()));
        assert_eq!(parser.option.value_header, vec!["H(1)".to_string(), "H(2)".to_string()]);
        assert_eq!(parser.option.preserve_empty_line, true);
    }

    #[test]
    fn test_simple_text_parser_parse() -> Result<(), anyhow::Error> {
        let input = r#"1           , 1(1),     1(2)
  1.1       , 1.1(1),   1.1(2)
  1.2       , 1.2(1),   1.2(2)
    1.2.1   , 1.2.1(1), 1.2.1(2)
"#;
        let options = SimpleTextParserOptions {
            indent: "  ".to_string(),
            delimiter: Some(",".to_string()),
            value_header: vec!["H(1)".to_string(), "H(2)".to_string()],
            ..Default::default()
        };
        let parser = SimpleTextParser::new(options);
        let outline = parser.parse(input)?;

        let mut expected_outline = reference_outline();
        expected_outline.key_header = Vec::new();
        assert_eq!(outline, expected_outline);

        let input_no_headers = r#"1           , 1(1),     1(2)
  1.1       , 1.1(1),   1.1(2)
  1.2       , 1.2(1),   1.2(2)
    1.2.1   , 1.2.1(1), 1.2.1(2)

"#;
        let options_no_headers = SimpleTextParserOptions {
            indent: "  ".to_string(),
            delimiter: Some(",".to_string()),
            ..Default::default()
        };
        let parser_no_headers = SimpleTextParser::new(options_no_headers);
        let outline_no_headers = parser_no_headers.parse(input_no_headers)?;

        let mut expected_outline_no_headers = reference_outline();
        expected_outline_no_headers.key_header = Vec::new();
        expected_outline_no_headers.value_header = Vec::new();
        assert_eq!(outline_no_headers, expected_outline_no_headers);

        let input_preserve_empty = r#"1

  1.1
"#;
        let options_preserve_empty = SimpleTextParserOptions {
            indent: "  ".to_string(),
            preserve_empty_line: true,
            key_header: vec!["H1".to_string()],
            value_header: vec!["V1".to_string()],
            ..Default::default()
        };
        let parser_preserve_empty = SimpleTextParser::new(options_preserve_empty);
        let outline_preserve_empty = parser_preserve_empty.parse(input_preserve_empty)?;

        let mut expected_outline_preserve_empty = Outline::new();
        expected_outline_preserve_empty.key_header = vec!["H1".to_string()];
        expected_outline_preserve_empty.value_header = vec!["V1".to_string()];
        expected_outline_preserve_empty.add_item("1", 1, Vec::new());
        expected_outline_preserve_empty.add_item("", 1, Vec::new());
        expected_outline_preserve_empty.add_item("1.1", 2, Vec::new());
        assert_eq!(outline_preserve_empty, expected_outline_preserve_empty);

        Ok(())
    }
}
