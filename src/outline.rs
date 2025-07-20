use thiserror::Error;

/// Represents errors that can occur during Outline validation.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum OutlineError {
    /// Indicates a validation failure with a descriptive message.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Represents a single item within an Outline structure.
///
/// An item consists of a key, a level (indentation), and a list of associated values.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutlineItem {
    /// The main key or title of the outline item.
    pub key: String,
    /// The indentation level of the item, starting from 1.
    pub level: u32,
    /// A list of additional values associated with the item.
    pub value: Vec<String>,
}

impl OutlineItem {
    /// Creates a new `OutlineItem`.
    ///
    /// # Arguments
    ///
    /// * `key` - The main key of the item.
    /// * `level` - The indentation level.
    /// * `value` - A vector of associated values.
    pub fn new(key: &str, level: u32, value: Vec<String>) -> Self {
        OutlineItem {
            key: key.to_string(),
            level,
            value,
        }
    }

    /// Validates the `OutlineItem`.
    ///
    /// Checks if the `level` is positive.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the item is valid, otherwise an `OutlineError`.
    pub fn validate(&self) -> Result<(), OutlineError> {
        if self.level == 0 {
            return Err(OutlineError::ValidationError(format!(
                "item level for item \"{}\" must be positive",
                self.key
            )));
        }
        Ok(())
    }

    /// Checks if the `OutlineItem` is valid.
    ///
    /// This is a convenience method that returns `true` if `validate` returns `Ok(())`,
    /// and `false` otherwise.
    pub fn valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Represents an entire Outline structure.
///
/// An outline consists of optional key and value headers, and a list of `OutlineItem`s.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Outline {
    /// Header for the keys, typically representing column names for different levels.
    pub key_header: Vec<String>,
    /// Header for the values, typically representing column names for additional data.
    pub value_header: Vec<String>,
    /// The list of `OutlineItem`s that form the content of the outline.
    pub item: Vec<OutlineItem>,
}

impl Outline {
    /// Creates a new, empty `Outline`.
    pub fn new() -> Self {
        Outline {
            key_header: Vec::new(),
            value_header: Vec::new(),
            item: Vec::new(),
        }
    }

    /// Adds a new `OutlineItem` to the outline.
    ///
    /// # Arguments
    ///
    /// * `key` - The main key of the item.
    /// * `level` - The indentation level.
    /// * `value` - A vector of associated values.
    pub fn add_item(&mut self, key: &str, level: u32, value: Vec<String>) {
        self.item.push(OutlineItem::new(key, level, value));
    }

    /// Validates the entire `Outline` structure.
    ///
    /// Checks if `key_header` and `value_header` elements are valid strings,
    /// and validates each `OutlineItem` within the outline.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the outline is valid, otherwise an `OutlineError`.
    pub fn validate(&self) -> Result<(), OutlineError> {
        if !self.key_header.iter().all(|v| v.is_empty() || v.is_ascii()) {
            return Err(OutlineError::ValidationError(
                "key_header elements must be strings.".to_string(),
            ));
        }
        if !self
            .value_header
            .iter()
            .all(|v| v.is_empty() || v.is_ascii())
        {
            return Err(OutlineError::ValidationError(
                "value_header elements must be strings.".to_string(),
            ));
        }
        for item in &self.item {
            item.validate()?;
        }
        Ok(())
    }

    /// Checks if the `Outline` is valid.
    ///
    /// This is a convenience method that returns `true` if `validate` returns `Ok(())`,
    /// and `false` otherwise.
    pub fn valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Calculates the maximum level (indentation) present in the outline.
    /// This includes the length of `key_header` and the levels of all `OutlineItem`s.
    pub fn max_level(&self) -> u32 {
        self.item
            .iter()
            .map(|item| item.level)
            .chain(std::iter::once(self.key_header.len() as u32))
            .max()
            .unwrap_or(0) // The `chain` ensures the iterator is never empty, guaranteeing a `Some` value from `max()`.
    }

    /// Calculates the maximum number of values associated with any item or in the value header.
    pub fn max_value_length(&self) -> usize {
        self.item
            .iter()
            .map(|item| item.value.len())
            .chain(std::iter::once(self.value_header.len()))
            .max()
            .unwrap_or(0) // The `chain` ensures the iterator is never empty, guaranteeing a `Some` value from `max()`
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outline_item_new() {
        let item = OutlineItem::new("key", 1, vec!["val1".to_string(), "val2".to_string()]);
        assert_eq!(item.key, "key");
        assert_eq!(item.level, 1);
        assert_eq!(item.value, vec!["val1".to_string(), "val2".to_string()]);
    }

    #[test]
    fn test_outline_item_validate() {
        let item = OutlineItem::new("key", 1, vec!["val1".to_string()]);
        assert!(item.validate().is_ok());

        let item = OutlineItem::new("key", 0, vec!["val1".to_string()]);
        assert!(item.validate().is_err());
        assert_eq!(
            item.validate().unwrap_err(),
            OutlineError::ValidationError(
                "item level for item \"key\" must be positive".to_string()
            )
        );
    }

    #[test]
    fn test_outline_item_valid() {
        let item = OutlineItem::new("key", 1, vec!["val1".to_string()]);
        assert!(item.valid());

        let item = OutlineItem::new("key", 0, vec!["val1".to_string()]);
        assert!(!item.valid());
    }

    #[test]
    fn test_outline_new() {
        let outline = Outline::new();
        assert!(outline.key_header.is_empty());
        assert!(outline.value_header.is_empty());
        assert!(outline.item.is_empty());
    }

    #[test]
    fn test_outline_add_item() {
        let mut outline = Outline::new();
        outline.add_item("key", 1, vec!["val1".to_string(), "val2".to_string()]);
        assert_eq!(outline.item.len(), 1);
        assert_eq!(outline.item[0].key, "key");
        assert_eq!(outline.item[0].level, 1);
        assert_eq!(
            outline.item[0].value,
            vec!["val1".to_string(), "val2".to_string()]
        );

        outline.add_item("key2", 1, vec!["val1".to_string(), "val2".to_string()]);
        assert_eq!(outline.item.len(), 2);
        assert_eq!(outline.item[0].key, "key");
        assert_eq!(outline.item[1].key, "key2");
    }

    #[test]
    fn test_outline_validate() {
        let mut outline = Outline::new();
        outline.key_header = vec!["H1".to_string(), "H2".to_string()];
        outline.value_header = vec!["V1".to_string(), "V2".to_string()];
        outline.add_item("key", 1, vec!["val1".to_string()]);
        assert!(outline.validate().is_ok());

        let mut invalid_outline = Outline::new();
        invalid_outline.key_header = vec!["H1".to_string(), "H2".to_string()];
        invalid_outline.value_header = vec!["V1".to_string(), "V2".to_string()];
        invalid_outline.add_item("key", 0, vec!["val1".to_string()]); // Invalid item
        assert!(invalid_outline.validate().is_err());
    }

    #[test]
    fn test_outline_valid() {
        let mut outline = Outline::new();
        outline.key_header = vec!["H1".to_string(), "H2".to_string()];
        outline.value_header = vec!["V1".to_string(), "V2".to_string()];
        outline.add_item("key", 1, vec!["val1".to_string()]);
        assert!(outline.valid());

        let mut invalid_outline = Outline::new();
        invalid_outline.key_header = vec!["H1".to_string(), "H2".to_string()];
        invalid_outline.value_header = vec!["V1".to_string(), "V2".to_string()];
        invalid_outline.add_item("key", 0, vec!["val1".to_string()]); // Invalid item
        assert!(!invalid_outline.valid());
    }

    #[test]
    fn test_outline_max_level() {
        let mut outline = Outline::new();
        outline.key_header = vec![];
        outline.add_item("1", 1, vec![]);
        assert_eq!(outline.max_level(), 1);

        outline.add_item("1.1", 2, vec![]);
        assert_eq!(outline.max_level(), 2);

        outline.key_header = vec!["H1".to_string(), "H2".to_string(), "H3".to_string()];
        assert_eq!(outline.max_level(), 3);
    }

    #[test]
    fn test_outline_max_value_length() {
        let mut outline = Outline::new();
        outline.value_header = vec![];
        outline.add_item("1", 1, vec![]);
        assert_eq!(outline.max_value_length(), 0);

        outline.add_item("1", 1, vec!["a".to_string()]);
        assert_eq!(outline.max_value_length(), 1);

        outline.add_item(
            "1",
            1,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        assert_eq!(outline.max_value_length(), 3);

        outline.value_header = vec!["H1".to_string(), "H2".to_string(), "H3".to_string()];
        assert_eq!(outline.max_value_length(), 3);
    }
}
