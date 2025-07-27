use crate::outline::Outline;
use anyhow::{Context, Result};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Options for configuring the `DirTreeParser`.
#[derive(Debug, Clone)]
pub struct DirTreeParserOptions {
    pub key_header: Option<String>,
    pub glob_pattern: Option<String>,
    pub dir_indicator: Option<String>,
}

impl Default for DirTreeParserOptions {
    fn default() -> Self {
        DirTreeParserOptions {
            key_header: None,
            glob_pattern: Some("**/*".to_string()),
            dir_indicator: None,
        }
    }
}

/// A parser for converting directory tree structure into an `Outline` structure.
pub struct DirTreeParser {
    option: DirTreeParserOptions,
}

impl DirTreeParser {
    /// Creates a new `DirTreeParser` with the given options.
    pub fn new(option: DirTreeParserOptions) -> Self {
        DirTreeParser { option }
    }

    /// Parses the input path and converts it into an `Outline` structure.
    pub fn parse(&self, input_path: &Path) -> Result<Outline> {
        let mut outline = Outline::new();

        // Parse key_header
        if let Some(s) = &self.option.key_header {
            outline.key_header = s.split(',').map(|s| s.trim().to_string()).collect();
        }
        let dir_indicator = self.option.dir_indicator.clone().unwrap_or("".to_string());

        // Construct the full glob pattern relative to the input_path
        let glob_pattern = self.option.glob_pattern.as_deref().unwrap_or("**/*");
        let full_glob_pattern = input_path.join(glob_pattern);
        let full_glob_pattern = full_glob_pattern.to_str().with_context(|| {
            format!(
                "Glob pattern path contains non-UTF-8 characters: {:?}",
                glob_pattern
            )
        })?;

        let mut outline_item = HashSet::new();
        for entry in glob::glob(full_glob_pattern)? {
            let path = entry?; // This `path` is now absolute
            let relative_path = path.strip_prefix(input_path)?;

            let mut current_path = PathBuf::new();
            for component in relative_path.components() {
                current_path.push(component);
                outline_item.insert(current_path.clone());
            }
        }

        let mut sorted_item: Vec<_> = outline_item.into_iter().collect();
        sorted_item.sort();

        for file_path in sorted_item {
            // Get basename
            let key_os_str = file_path
                .file_name()
                .with_context(|| format!("Path has no filename: {:?}", file_path))?;
            let key = key_os_str
                .to_str()
                .with_context(|| format!("Filename is not valid UTF-8: {:?}", key_os_str))?;
            let mut key_with_indicator = key.to_string();

            let full_path = input_path.join(&file_path);
            if full_path.is_dir() {
                key_with_indicator.push_str(&dir_indicator);
            }

            // Level is based on the number of components in the relative path
            let level = file_path.components().count() as u32;

            outline.add_item(&key_with_indicator, level, vec![]);
        }

        Ok(outline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_dir_tree_parser_options_default() {
        let options = DirTreeParserOptions::default();
        assert_eq!(options.key_header, None);
        assert_eq!(options.glob_pattern, Some("**/*".to_string()));
        assert_eq!(options.dir_indicator, None);
    }

    #[test]
    fn test_dir_tree_parser_new() {
        let options = DirTreeParserOptions {
            key_header: Some("Header1,Header2".to_string()),
            glob_pattern: Some("*.txt".to_string()),
            dir_indicator: Some("/".to_string()),
        };
        let parser = DirTreeParser::new(options.clone());
        assert_eq!(parser.option.key_header, options.key_header);
        assert_eq!(parser.option.glob_pattern, options.glob_pattern);
        assert_eq!(parser.option.dir_indicator, options.dir_indicator);
    }

    #[test]
    fn test_dir_tree_parser_parse_empty_dir() -> Result<()> {
        let tmp_dir = tempdir()?;
        let options = DirTreeParserOptions::default();
        let parser = DirTreeParser::new(options);
        let outline = parser.parse(tmp_dir.path())?;

        assert!(outline.item.is_empty());
        Ok(())
    }

    #[test]
    fn test_dir_tree_parser_parse_simple_dir() -> Result<()> {
        let tmp_dir = tempdir()?;
        fs::create_dir_all(tmp_dir.path().join("subdir1"))?;
        fs::write(tmp_dir.path().join("file1.txt"), "content")?;
        fs::write(tmp_dir.path().join("subdir1/file2.txt"), "content")?;

        let options = DirTreeParserOptions::default();
        let parser = DirTreeParser::new(options);
        let outline = parser.parse(tmp_dir.path())?;

        // Expected items (order might vary slightly depending on OS/filesystem, but levels should be correct)
        // We'll check for existence and correct level/key
        let mut expected_items = vec![
            ("file1.txt".to_string(), 1),
            ("subdir1".to_string(), 1),
            ("file2.txt".to_string(), 2),
        ];
        // Add dir_indicator for directories
        if let Some(indicator) = &parser.option.dir_indicator {
            if !indicator.is_empty() {
                for (key, _level) in &mut expected_items {
                    if key == "subdir1" {
                        *key = format!("{}{}", key, indicator);
                    }
                }
            }
        }

        // Convert outline items to a comparable format (key, level)
        let actual_items: Vec<(String, u32)> = outline
            .item
            .iter()
            .map(|item| (item.key.clone(), item.level))
            .collect();

        // Sort both for consistent comparison
        let mut actual_sorted = actual_items;
        actual_sorted.sort_by(|a, b| a.0.cmp(&b.0));
        expected_items.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(actual_sorted.len(), expected_items.len());
        for (actual, expected) in actual_sorted.iter().zip(expected_items.iter()) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
        }

        Ok(())
    }

    #[test]
    fn test_dir_tree_parser_parse_with_glob_and_indicator() -> Result<()> {
        let tmp_dir = tempdir()?;
        fs::create_dir_all(tmp_dir.path().join("subdir1"))?;
        fs::write(tmp_dir.path().join("file1.txt"), "content")?;
        fs::write(tmp_dir.path().join("file2.log"), "content")?;
        fs::write(tmp_dir.path().join("subdir1/file3.txt"), "content")?;

        let options = DirTreeParserOptions {
            key_header: None,
            glob_pattern: Some("**/*.txt".to_string()),
            dir_indicator: Some("/".to_string()),
        };
        let parser = DirTreeParser::new(options);
        let outline = parser.parse(tmp_dir.path())?;

        let mut expected_items = vec![
            ("file1.txt".to_string(), 1),
            ("subdir1/".to_string(), 1),
            ("file3.txt".to_string(), 2),
        ];

        let actual_items: Vec<(String, u32)> = outline
            .item
            .iter()
            .map(|item| (item.key.clone(), item.level))
            .collect();

        let mut actual_sorted = actual_items;
        actual_sorted.sort_by(|a, b| a.0.cmp(&b.0));
        expected_items.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(actual_sorted.len(), expected_items.len());
        for (actual, expected) in actual_sorted.iter().zip(expected_items.iter()) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
        }

        Ok(())
    }
}
