// src/parser/html_list.rs
use clap::Args;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

#[derive(Debug, Args)]
pub struct HtmlListParserOptions {
    /// key header
    #[arg(long, default_values_t = Vec::<String>::new())]
    pub key_header: Vec<String>,
}

/// A parser for HTML lists that extracts list items and their hierarchy.
pub struct HtmlListParser {
    /// Options for the HTML list parser.
    options: HtmlListParserOptions,
}

impl HtmlListParser {
    /// Creates a new instance of `HtmlListParser` with the given options.
    ///
    /// # Arguments
    /// * `options` - An instance of `HtmlListParserOptions` containing configuration for the
    ///   parser.
    pub fn new(options: HtmlListParserOptions) -> Self {
        HtmlListParser { options }
    }

    /// Parses the given HTML input and returns an `Outline`.
    ///
    /// # Arguments
    /// * `input` - A string slice containing the HTML input to be parsed.
    ///
    /// # Returns
    /// * `Ok(Outline)` - If parsing is successful, returns an `Outline` containing the parsed list
    ///   items.
    pub fn parse(&self, input: &str) -> anyhow::Result<crate::outline::Outline> {
        let mut outline = crate::outline::Outline::new();
        outline.key_header = self.options.key_header.clone();
        outline.value_header = Vec::new();

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut input.as_bytes())?;

        Self::traverse_and_parse(&dom.document, 0, &mut outline);

        Ok(outline)
    }

    /// Recursively traverses the DOM tree and parses list items.
    ///
    /// # Arguments
    /// * `handle` - The current node in the DOM tree.
    /// * `level` - The current level of nesting in the list.
    /// * `outline` - The outline to which the parsed items will be added.
    fn traverse_and_parse(handle: &Handle, level: u32, outline: &mut crate::outline::Outline) {
        let node = handle;
        let mut level = level;

        if let NodeData::Element { name, .. } = &node.data {
            let tag = name.local.as_ref();

            match tag {
                "ul" | "ol" => {
                    level += 1;
                }
                "li" => {
                    let text = Self::extract_text_nonlist(node);
                    outline.add_item(text.trim(), level, Vec::new());
                }
                _ => {}
            }
        }

        for child in node.children.borrow().iter() {
            Self::traverse_and_parse(child, level, outline);
        }
    }

    /// Extracts the text content from a node, handling nested elements.
    fn extract_text_nonlist(handle: &Handle) -> String {
        let mut result = String::new();
        for child in handle.children.borrow().iter() {
            match &child.data {
                NodeData::Text { contents } => {
                    let text = contents.borrow();
                    result.push_str(text.trim());
                }
                NodeData::Element { name, .. } => {
                    let tag = name.local.as_ref();
                    if tag != "ul" && tag != "ol" {
                        let inner = Self::extract_text_nonlist(child);
                        result.push_str(&inner);
                    }
                }
                _ => {}
            }
        }
        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_ul() {
        let html_input = "<ul><li>Item 1</li><li>Item 2<ul><li>Subitem 2.1</li></ul></li></ul>";
        let options = HtmlListParserOptions { key_header: vec![] };
        let parser = HtmlListParser::new(options);
        let outline = parser.parse(html_input).unwrap();

        assert_eq!(outline.item.len(), 3);
        assert_eq!(outline.item[0].key, "Item 1");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[1].key, "Item 2");
        assert_eq!(outline.item[1].level, 1);
        assert_eq!(outline.item[2].key, "Subitem 2.1");
        assert_eq!(outline.item[2].level, 2);
    }

    #[test]
    fn test_empty_input() {
        let html_input = "";
        let options = HtmlListParserOptions { key_header: vec![] };
        let parser = HtmlListParser::new(options);
        let outline = parser.parse(html_input).unwrap();

        assert_eq!(outline.item.len(), 0);
    }

    #[test]
    fn test_nested_ol() {
        let html_input =
            "<ol><li>One<ol><li>One.One</li><li>One.Two</li></ol></li><li>Two</li></ol>";
        let options = HtmlListParserOptions { key_header: vec![] };
        let parser = HtmlListParser::new(options);
        let outline = parser.parse(html_input).unwrap();

        assert_eq!(outline.item.len(), 4);
        assert_eq!(outline.item[0].key, "One");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[1].key, "One.One");
        assert_eq!(outline.item[1].level, 2);
        assert_eq!(outline.item[2].key, "One.Two");
        assert_eq!(outline.item[2].level, 2);
        assert_eq!(outline.item[3].key, "Two");
        assert_eq!(outline.item[3].level, 1);
    }

    #[test]
    fn test_li_with_other_tags() {
        let html_input = "<ul><li><b>Bold Item</b></li><li><p>Paragraph Item</p></li></ul>";
        let options = HtmlListParserOptions { key_header: vec![] };
        let parser = HtmlListParser::new(options);
        let outline = parser.parse(html_input).unwrap();

        assert_eq!(outline.item.len(), 2);
        assert_eq!(outline.item[0].key, "Bold Item");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[1].key, "Paragraph Item");
        assert_eq!(outline.item[1].level, 1);
    }

    #[test]
    fn test_key_header_option() {
        let html_input = "<ul><li>Item 1</li></ul>";
        let options = HtmlListParserOptions {
            key_header: vec!["Header1".to_string(), "Header2".to_string()],
        };
        let parser = HtmlListParser::new(options);
        let outline = parser.parse(html_input).unwrap();

        assert_eq!(
            outline.key_header,
            vec!["Header1".to_string(), "Header2".to_string()]
        );
    }
}
