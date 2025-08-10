use std::cell::RefCell;
use std::rc::{Rc, Weak};
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
#[derive(Debug, PartialEq, Eq, Clone, Default)]
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
#[derive(Debug, PartialEq, Eq, Clone, Default)]
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
        Self::default()
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

    /// Converts the `Outline` into a hierarchical `OutlineTree` structure.
    ///
    /// This method builds a tree where each node represents an `OutlineItem`
    /// and its children are items that are indented under it.
    ///
    /// # Returns
    ///
    /// An `Rc<RefCell<OutlineTree>>` representing the root of the constructed tree.
    pub fn to_tree(&self) -> Rc<RefCell<OutlineTree>> {
        let root = OutlineTree::new_root();
        let mut last_node_rc = Rc::clone(&root);

        for item in &self.item {
            let mut parent_node_rc = Rc::clone(&root);

            if item.level > 1 {
                let last_node_borrow = last_node_rc.borrow();
                if let Some(last_node_item) = last_node_borrow.item() {
                    if item.level > last_node_item.level {
                        parent_node_rc = Rc::clone(&last_node_rc);
                    } else {
                        let mut current_search_node_rc = Rc::clone(&last_node_rc);
                        loop {
                            let parent_option = {
                                let current_search_node_borrow = current_search_node_rc.borrow();
                                if current_search_node_borrow.is_root() {
                                    break;
                                }
                                if let Some(current_search_node_item) =
                                    current_search_node_borrow.item()
                                {
                                    if current_search_node_item.level < item.level {
                                        break;
                                    }
                                }
                                current_search_node_borrow.parent.upgrade()
                            }; // current_search_node_borrow is dropped here

                            if let Some(p) = parent_option {
                                current_search_node_rc = p;
                            } else {
                                break; // Should not happen for non-root nodes
                            }
                        }
                        parent_node_rc = current_search_node_rc;
                    }
                }
            }
            let new_node_rc = OutlineTree::add_child(&parent_node_rc, item.clone());
            last_node_rc = new_node_rc;
        }
        root
    }
}

/// Represents a node in the hierarchical `OutlineTree`.
///
/// Each node contains an `OutlineItem` (if not the root), a weak reference to its parent,
/// and a list of its children.
#[derive(Debug)]
pub struct OutlineTree {
    /// The `OutlineItem` associated with this node. `None` for the root node.
    item: Option<OutlineItem>,
    /// A weak reference to the parent node. Used to prevent reference cycles.
    parent: Weak<RefCell<OutlineTree>>,
    /// A list of child nodes.
    children: Vec<Rc<RefCell<OutlineTree>>>,
}

impl OutlineTree {
    /// Creates a new root `OutlineTree` node.
    pub fn new_root() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(OutlineTree {
            item: None,
            parent: Weak::new(),
            children: Vec::new(),
        }))
    }

    /// Creates a new `OutlineTree` node with a parent.
    ///
    /// # Arguments
    ///
    /// * `item` - The `OutlineItem` for this node.
    /// * `parent_rc` - An `Rc<RefCell<OutlineTree>>` to the parent node.
    pub fn new_with_parent(
        item: OutlineItem,
        parent_rc: &Rc<RefCell<OutlineTree>>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(OutlineTree {
            item: Some(item),
            parent: Rc::downgrade(parent_rc),
            children: Vec::new(),
        }))
    }

    /// Checks if this node is the root of the tree.
    pub fn is_root(&self) -> bool {
        self.parent.upgrade().is_none() && self.item.is_none()
    }

    /// Adds a new child node to the given parent node.
    ///
    /// # Arguments
    ///
    /// * `parent_rc` - The `Rc<RefCell<OutlineTree>>` of the parent node to which the child will be added.
    /// * `item` - The `OutlineItem` for the new child node.
    ///
    /// # Returns
    ///
    /// An `Rc<RefCell<OutlineTree>>` representing the newly added child node.
    pub fn add_child(
        parent_rc: &Rc<RefCell<OutlineTree>>,
        item: OutlineItem,
    ) -> Rc<RefCell<OutlineTree>> {
        let child = OutlineTree::new_with_parent(item, parent_rc);
        parent_rc.borrow_mut().children.push(Rc::clone(&child));
        child
    }

    /// Returns the parent node, if it exists.
    pub fn parent(&self) -> Option<Rc<RefCell<OutlineTree>>> {
        self.parent.upgrade()
    }

    /// Returns a reference to the `OutlineItem` of this node.
    pub fn item(&self) -> Option<&OutlineItem> {
        self.item.as_ref()
    }

    /// Returns a reference to the children vector of this node.
    pub fn children(&self) -> &Vec<Rc<RefCell<OutlineTree>>> {
        &self.children
    }

    /// Checks if this node is a leaf node (i.e., has no children).
    pub fn is_leaf(&self) -> bool {
        !self.is_root() && self.children.is_empty()
    }

    /// Returns an iterator over all descendant nodes in pre-order (children left-to-right).
    pub fn descendants(rc: &Rc<RefCell<OutlineTree>>) -> Descendants {
        let mut stack = Vec::new();
        for child in rc.borrow().children().iter().rev() {
            stack.push(child.clone());
        }
        Descendants { stack }
    }

    /// Returns an iterator over ancestors from parent to root (excluding self).
    pub fn ancestors(rc: &Rc<RefCell<OutlineTree>>) -> Ancestors {
        Ancestors {
            current: rc.borrow().parent(), // Start from the parent
        }
    }

    /// Returns previous sibling if exists.
    pub fn prev(rc: &Rc<RefCell<OutlineTree>>) -> Option<Rc<RefCell<OutlineTree>>> {
        let parent = rc.borrow().parent()?;
        let siblings = parent.borrow();
        let ptr = Rc::as_ptr(rc);
        let idx = siblings
            .children()
            .iter()
            .position(|s| Rc::as_ptr(s) == ptr)?;
        if idx == 0 {
            None
        } else {
            Some(Rc::clone(&siblings.children()[idx - 1]))
        }
    }

    /// Returns next sibling if exists.
    pub fn next(rc: &Rc<RefCell<OutlineTree>>) -> Option<Rc<RefCell<OutlineTree>>> {
        let parent = rc.borrow().parent()?;
        let siblings = parent.borrow();
        let ptr = Rc::as_ptr(rc);
        let idx = siblings
            .children()
            .iter()
            .position(|s| Rc::as_ptr(s) == ptr)?;
        if idx + 1 >= siblings.children().len() {
            None
        } else {
            Some(Rc::clone(&siblings.children()[idx + 1]))
        }
    }
}

/// An iterator over the ancestors of an `OutlineTree` node.
pub struct Ancestors {
    current: Option<Rc<RefCell<OutlineTree>>>,
}

impl Iterator for Ancestors {
    type Item = Rc<RefCell<OutlineTree>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_node = self.current.take()?; // Take the current node (which is the next ancestor)
        self.current = next_node.borrow().parent(); // Set the next ancestor to its parent
        Some(next_node)
    }
}

/// An iterator over the descendants of an `OutlineTree` node.
pub struct Descendants {
    stack: Vec<Rc<RefCell<OutlineTree>>>,
}

impl Iterator for Descendants {
    type Item = Rc<RefCell<OutlineTree>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_node = self.stack.pop()?; // Get the next node from the stack

        // Add children to the stack in reverse order to process left-to-right
        for child in next_node.borrow().children().iter().rev() {
            self.stack.push(child.clone());
        }

        Some(next_node)
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

    #[test]
    fn test_outline_to_tree_empty() {
        let outline = Outline::new();
        let tree = outline.to_tree();
        assert!(tree.borrow().is_root());
        assert!(tree.borrow().children().is_empty());
    }

    #[test]
    fn test_outline_to_tree_single_item() {
        let mut outline = Outline::new();
        outline.add_item("Item 1", 1, vec![]);
        let tree = outline.to_tree();

        let tree_borrow = tree.borrow();
        assert!(tree_borrow.is_root());
        assert_eq!(tree_borrow.children().len(), 1);

        let child1_rc = &tree_borrow.children()[0];
        let child1 = child1_rc.borrow();
        assert_eq!(child1.item().unwrap().key, "Item 1");
        assert_eq!(child1.item().unwrap().level, 1);
        assert!(child1.is_leaf());
        assert_eq!(child1.parent().unwrap().borrow().item(), None);
    }

    #[test]
    fn test_outline_to_tree_nested_items() {
        let mut outline = Outline::new();
        outline.add_item("Item 1", 1, vec![]);
        outline.add_item("Item 1.1", 2, vec![]);
        outline.add_item("Item 1.2", 2, vec![]);
        outline.add_item("Item 1.2.1", 3, vec![]);
        outline.add_item("Item 2", 1, vec![]);

        let tree = outline.to_tree();

        // Root
        let tree_borrow = tree.borrow();
        assert!(tree_borrow.is_root());
        assert_eq!(tree_borrow.children().len(), 2);

        // Item 1
        let item1_rc = Rc::clone(&tree_borrow.children()[0]);
        let item1 = item1_rc.borrow();
        assert_eq!(item1.item().unwrap().key, "Item 1");
        assert_eq!(item1.item().unwrap().level, 1);
        assert_eq!(item1.children().len(), 2);
        assert_eq!(item1.parent().unwrap().borrow().item(), None);

        // Item 1.1
        let item1_1_rc = Rc::clone(&item1.children()[0]);
        let item1_1 = item1_1_rc.borrow();
        assert_eq!(item1_1.item().unwrap().key, "Item 1.1");
        assert_eq!(item1_1.item().unwrap().level, 2);
        assert!(item1_1.is_leaf());
        assert_eq!(
            item1_1.parent().unwrap().borrow().item().unwrap().key,
            "Item 1"
        );

        // Item 1.2
        let item1_2_rc = Rc::clone(&item1.children()[1]);
        let item1_2 = item1_2_rc.borrow();
        assert_eq!(item1_2.item().unwrap().key, "Item 1.2");
        assert_eq!(item1_2.item().unwrap().level, 2);
        assert_eq!(item1_2.children().len(), 1);
        assert_eq!(
            item1_2.parent().unwrap().borrow().item().unwrap().key,
            "Item 1"
        );

        // Item 1.2.1
        let item1_2_1_rc = Rc::clone(&item1_2.children()[0]);
        let item1_2_1 = item1_2_1_rc.borrow();
        assert_eq!(item1_2_1.item().unwrap().key, "Item 1.2.1");
        assert_eq!(item1_2_1.item().unwrap().level, 3);
        assert!(item1_2_1.is_leaf());
        assert_eq!(
            item1_2_1.parent().unwrap().borrow().item().unwrap().key,
            "Item 1.2"
        );

        // Item 2
        let item2_rc = Rc::clone(&tree_borrow.children()[1]);
        let item2 = item2_rc.borrow();
        assert_eq!(item2.item().unwrap().key, "Item 2");
        assert_eq!(item2.item().unwrap().level, 1);
        assert!(item2.is_leaf());
        assert_eq!(item2.parent().unwrap().borrow().item(), None);
    }

    #[test]
    fn test_outline_to_tree_complex_levels() {
        let mut outline = Outline::new();
        outline.add_item("A", 1, vec![]);
        outline.add_item("B", 2, vec![]);
        outline.add_item("C", 3, vec![]);
        outline.add_item("D", 2, vec![]);
        outline.add_item("E", 1, vec![]);
        outline.add_item("F", 2, vec![]);

        let tree = outline.to_tree();
        let tree_borrow = tree.borrow();

        // A
        let a_node = Rc::clone(&tree_borrow.children()[0]);
        let a_node_borrow = a_node.borrow();
        assert_eq!(a_node_borrow.item().unwrap().key, "A");
        assert_eq!(a_node_borrow.children().len(), 2);

        // B
        let b_node = Rc::clone(&a_node_borrow.children()[0]);
        let b_node_borrow = b_node.borrow();
        assert_eq!(b_node_borrow.item().unwrap().key, "B");
        assert_eq!(b_node_borrow.children().len(), 1);

        // C
        let c_node = Rc::clone(&b_node_borrow.children()[0]);
        let c_node_borrow = c_node.borrow();
        assert_eq!(c_node_borrow.item().unwrap().key, "C");
        assert!(c_node_borrow.is_leaf());

        // D
        let d_node = Rc::clone(&a_node_borrow.children()[1]);
        let d_node_borrow = d_node.borrow();
        assert_eq!(d_node_borrow.item().unwrap().key, "D");
        assert!(d_node_borrow.is_leaf());

        // E
        let e_node = Rc::clone(&tree_borrow.children()[1]);
        let e_node_borrow = e_node.borrow();
        assert_eq!(e_node_borrow.item().unwrap().key, "E");
        assert_eq!(e_node_borrow.children().len(), 1);

        // F
        let f_node = Rc::clone(&e_node_borrow.children()[0]);
        let f_node_borrow = f_node.borrow();
        assert_eq!(f_node_borrow.item().unwrap().key, "F");
        assert!(f_node_borrow.is_leaf());
    }

    #[test]
    fn test_outline_tree_is_root() {
        let root = OutlineTree::new_root();
        assert!(root.borrow().is_root());

        let child = OutlineTree::new_with_parent(OutlineItem::new("child", 1, vec![]), &root);
        assert!(!child.borrow().is_root());
    }

    #[test]
    fn test_outline_tree_parent() {
        let root = OutlineTree::new_root();
        let child = OutlineTree::new_with_parent(OutlineItem::new("child", 1, vec![]), &root);

        assert!(root.borrow().parent().is_none());
        assert_eq!(child.borrow().parent().unwrap().borrow().item(), None);
    }

    #[test]
    fn test_outline_tree_add_child() {
        let root = OutlineTree::new_root();
        let child = OutlineTree::add_child(&root, OutlineItem::new("child", 1, vec![]));

        assert_eq!(root.borrow().children().len(), 1);
        assert_eq!(
            root.borrow().children()[0].borrow().item().unwrap().key,
            "child"
        );
        assert_eq!(child.borrow().parent().unwrap().borrow().item(), None);
        assert!(child.borrow().is_leaf());
    }

    #[test]
    fn test_outline_tree_is_leaf() {
        let root = OutlineTree::new_root();
        assert!(!root.borrow().is_leaf()); // Root is not a leaf initially

        let child = OutlineTree::add_child(&root, OutlineItem::new("child", 1, vec![]));
        assert!(child.borrow().is_leaf()); // Child is a leaf

        let grand_child = OutlineTree::add_child(&child, OutlineItem::new("grandchild", 2, vec![]));
        assert!(!child.borrow().is_leaf()); // Child is no longer a leaf
        assert!(grand_child.borrow().is_leaf()); // Grandchild is a leaf
    }

    #[test]
    fn test_descendants_ancestors_prev_next() {
        // Build tree: root -> A -> (B -> C), and A -> D; root -> E -> F
        let mut outline = Outline::new();
        outline.add_item("A", 1, vec![]);
        outline.add_item("B", 2, vec![]);
        outline.add_item("C", 3, vec![]);
        outline.add_item("D", 2, vec![]);
        outline.add_item("E", 1, vec![]);
        outline.add_item("F", 2, vec![]);

        let tree = outline.to_tree();
        let a = tree.borrow().children()[0].clone();
        let e = tree.borrow().children()[1].clone();

        let b = a.borrow().children()[0].clone();
        let d = a.borrow().children()[1].clone();
        let c = b.borrow().children()[0].clone();

        // descendants of A: [B, C, D]
        let desc_a: Vec<_> = OutlineTree::descendants(&a).collect();
        assert_eq!(desc_a.len(), 3);
        assert_eq!(desc_a[0].borrow().item().unwrap().key, "B");
        assert_eq!(desc_a[1].borrow().item().unwrap().key, "C");
        assert_eq!(desc_a[2].borrow().item().unwrap().key, "D");

        // ancestors of C: [B, A, root]
        let anc_c: Vec<_> = OutlineTree::ancestors(&c).collect();
        assert_eq!(anc_c.len(), 3);
        assert_eq!(anc_c[0].borrow().item().unwrap().key, "B");
        assert_eq!(anc_c[1].borrow().item().unwrap().key, "A");
        assert!(anc_c[2].borrow().is_root());

        // prev/next among siblings under A
        assert!(OutlineTree::prev(&b).is_none());
        assert_eq!(
            OutlineTree::next(&b).unwrap().borrow().item().unwrap().key,
            "D"
        );
        assert_eq!(
            OutlineTree::prev(&d).unwrap().borrow().item().unwrap().key,
            "B"
        );
        assert!(OutlineTree::next(&d).is_none());

        // prev/next across top-level siblings: A <-> E
        assert!(OutlineTree::prev(&a).is_none());
        assert_eq!(
            OutlineTree::next(&a).unwrap().borrow().item().unwrap().key,
            "E"
        );
        assert_eq!(
            OutlineTree::prev(&e).unwrap().borrow().item().unwrap().key,
            "A"
        );
        assert!(OutlineTree::next(&e).is_none());
    }
}
