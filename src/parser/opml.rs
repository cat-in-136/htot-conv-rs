use anyhow::Result;
use clap::Args;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::outline::Outline;

#[derive(Debug, Args)]
pub struct OpmlParserOptions {
    /// key header
    #[arg(long, default_values_t = Vec::<String>::new())]
    pub key_header: Vec<String>,
    /// value header
    #[arg(long, default_values_t = Vec::<String>::new())]
    pub value_header: Vec<String>,
}

pub struct OpmlParser {
    options: OpmlParserOptions,
}

impl OpmlParser {
    pub fn new(options: OpmlParserOptions) -> Self {
        OpmlParser { options }
    }

    pub fn parse(&self, input: &str) -> Result<Outline> {
        let mut outline = Outline::new();
        outline.key_header = self.options.key_header.clone();
        outline.value_header = self.options.value_header.clone();

        let mut reader = Reader::from_str(input);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut outline_level = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Error at position {}: {:?}",
                        reader.buffer_position(),
                        e
                    ))
                }
                Ok(Event::Eof) => break,
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"outline" => {
                    // Determine the current level based on the stack
                    outline_level += 1;
                    self.generate_outline_item(&mut outline, &e.attributes(), outline_level)?;
                }
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"outline" => {
                    self.generate_outline_item(&mut outline, &e.attributes(), outline_level + 1)?;
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"outline" => {
                    outline_level -= 1;
                }
                _ => (),
            }
            buf.clear();
        }
        Ok(outline)
    }

    fn generate_outline_item(
        &self,
        outline: &mut Outline,
        attributes: &Attributes,
        level: u32,
    ) -> Result<()> {
        let mut text = String::new();
        let mut item_values = vec![String::new(); outline.value_header.len()];

        for attr in attributes.clone().into_iter() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.into_inner()).into_owned();
            let value = attr.unescape_value()?.into_owned();

            if key == "text" {
                text = value.trim().to_string();
            } else if let Some(value_pos) = outline.value_header.iter().position(|x| x == &key) {
                item_values[value_pos] = value.to_string();
            }
        }

        outline.add_item(&text, level, item_values);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_opml() {
        let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="1.0">
    <head>
        <title>My Outline</title>
    </head>
    <body>
        <outline text="Item 1">
            <outline text="Subitem 1.1"/>
            <outline text="Subitem 1.2"/>
        </outline>
        <outline text="Item 2"/>
    </body>
</opml>
"#;
        let options = OpmlParserOptions {
            key_header: vec![],
            value_header: vec![],
        };
        let parser = OpmlParser::new(options);
        let outline = parser.parse(xml_input).unwrap();

        assert_eq!(outline.item.len(), 4);
        assert_eq!(outline.item[0].key, "Item 1");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[1].key, "Subitem 1.1");
        assert_eq!(outline.item[1].level, 2);
        assert_eq!(outline.item[2].key, "Subitem 1.2");
        assert_eq!(outline.item[2].level, 2);
        assert_eq!(outline.item[3].key, "Item 2");
        assert_eq!(outline.item[3].level, 1);
    }

    #[test]
    fn test_opml_with_attributes() {
        let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="1.0">
    <body>
        <outline text="Task A" _note="Note for Task A" due="2025-01-01"/>
        <outline text="Task B" priority="high"/>
    </body>
</opml>
"#;
        let options = OpmlParserOptions {
            key_header: vec![],
            value_header: vec!["due".to_string(), "priority".to_string()],
        };
        let parser = OpmlParser::new(options);
        let outline = parser.parse(xml_input).unwrap();

        assert_eq!(outline.item.len(), 2);
        assert_eq!(outline.item[0].key, "Task A");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[0].value.len(), 2);
        assert_eq!(outline.item[0].value[0], "2025-01-01");
        assert_eq!(outline.item[0].value[1], "");
        assert_eq!(outline.item[1].key, "Task B");
        assert_eq!(outline.item[1].level, 1);
        assert_eq!(outline.item[1].value.len(), 2);
        assert_eq!(outline.item[1].value[0], "");
        assert_eq!(outline.item[1].value[1], "high");

        assert_eq!(outline.value_header.len(), 2);
        assert_eq!(outline.value_header[0], "due");
        assert_eq!(outline.value_header[1], "priority");
    }
}
