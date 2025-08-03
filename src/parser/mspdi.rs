use anyhow::Result;
use anyhow::anyhow;
use clap::Args;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

use crate::outline::Outline;

#[derive(Debug, Args)]
pub struct MspdiParserOptions {
    /// key header
    #[arg(long, default_values_t = Vec::<String>::new())]
    pub key_header: Vec<String>,
    /// value header
    #[arg(long, default_values_t = Vec::<String>::new(), value_delimiter = ',')]
    pub value_header: Vec<String>,
}

pub struct MspdiParser {
    options: MspdiParserOptions,
}

impl MspdiParser {
    pub fn new(options: MspdiParserOptions) -> Self {
        MspdiParser { options }
    }

    pub fn parse(&self, input: &str) -> Result<Outline> {
        let mut outline = Outline::new();
        outline.key_header = self.options.key_header.clone();
        outline.value_header = self.options.value_header.clone();

        let mut reader = Reader::from_str(input);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut breadcrumb: Vec<String> = Vec::new();
        let mut current_task_values: HashMap<String, String> = HashMap::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => return Err(anyhow!("Error at position {}: {:?}", reader.buffer_position(), e)),
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().into_inner()).into_owned();
                    breadcrumb.push(tag_name.clone());
                    if tag_name == "Task" {
                        current_task_values.clear();
                    }
                }
                Ok(Event::End(e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().into_inner()).into_owned();
                    breadcrumb.pop();
                    if tag_name == "Task" {
                        self.generate_outline_item(&mut outline, &current_task_values);
                    }
                }
                Ok(Event::Text(e)) => {
                    if breadcrumb.contains(&"Task".to_string()) {
                        let text = e.unescape()?.into_owned();
                        if let Some(last_tag) = breadcrumb.last() {
                            current_task_values
                                .entry(last_tag.clone())
                                .or_insert_with(String::new)
                                .push_str(&text);
                        }
                    }
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
        values: &HashMap<String, String>,
    ) {
        let mut text = String::new();
        let mut level = 1;
        let mut item_values: Vec<String> = vec!["".to_string(); outline.value_header.len()];

        for (key, val) in values.iter() {
            if key == "Name" {
                text = val.clone();
            } else if key == "OutlineLevel" {
                level = val.parse::<u32>().unwrap_or(1);
            } else if let Some(index) = outline.value_header.iter().position(|h| h == key) {
                item_values[index] = val.clone();
            }
        }
        outline.add_item(&text, level, item_values);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mspdi() {
        let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<Project>
    <Tasks>
        <Task>
            <UID>1</UID>
            <Name>Task 1</Name>
            <OutlineLevel>1</OutlineLevel>
        </Task>
        <Task>
            <UID>2</UID>
            <Name>Subtask 1.1</Name>
            <OutlineLevel>2</OutlineLevel>
        </Task>
        <Task>
            <UID>3</UID>
            <Name>Task 2</Name>
            <OutlineLevel>1</OutlineLevel>
        </Task>
    </Tasks>
</Project>
"#;
        let options = MspdiParserOptions {
            key_header: vec![],
            value_header: vec![],
        };
        let parser = MspdiParser::new(options);
        let outline = parser.parse(xml_input).unwrap();

        assert_eq!(outline.item.len(), 3);
        assert_eq!(outline.item[0].key, "Task 1");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[1].key, "Subtask 1.1");
        assert_eq!(outline.item[1].level, 2);
        assert_eq!(outline.item[2].key, "Task 2");
        assert_eq!(outline.item[2].level, 1);
    }

    #[test]
    fn test_mspdi_with_values() {
        let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<Project>
    <Tasks>
        <Task>
            <UID>1</UID>
            <Name>Task A</Name>
            <OutlineLevel>1</OutlineLevel>
            <StartDate>2025-01-01</StartDate>
            <FinishDate>2025-01-05</FinishDate>
        </Task>
        <Task>
            <UID>2</UID>
            <Name>Task B</Name>
            <OutlineLevel>1</OutlineLevel>
            <StartDate>2025-01-06</StartDate>
            <FinishDate>2025-01-10</FinishDate>
        </Task>
    </Tasks>
</Project>
"#;
        let options = MspdiParserOptions {
            key_header: vec![],
            value_header: vec!["StartDate".to_string(), "FinishDate".to_string()],
        };
        let parser = MspdiParser::new(options);
        let outline = parser.parse(xml_input).unwrap();

        assert_eq!(outline.item.len(), 2);
        assert_eq!(outline.item[0].key, "Task A");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(outline.item[0].value[0], "2025-01-01");
        assert_eq!(outline.item[0].value[1], "2025-01-05");
        assert_eq!(outline.item[1].key, "Task B");
        assert_eq!(outline.item[1].level, 1);
        assert_eq!(outline.item[1].value[0], "2025-01-06");
        assert_eq!(outline.item[1].value[1], "2025-01-10");
    }
}
