//! # htot-conv-rs - Hierarchical-Tree Outline Text Converter (Rust Port)
//! 
//! `htot-conv-rs` is a Rust re-implementation of the `htot_conv` Ruby project.
//! The primary goals of this port are to enhance performance, ensure memory safety,
//! and provide a more robust command-line interface (CLI) application.
//! 
//! ## Types of Input
//! 
//! ### `simple_text`
//! 
//! A text file consisting of multiple lines where:
//! - `<line> ::= { <indent> } <key> { <delimiter> <value> }`
//! - `<key>` : a text that does not start with `<indent>` and does not contain `<delimiter>` (if `<delimiter>` specified).
//! - `<value>` : a text that does not contain `<delimiter>`.
//! - `<indent>` : specified by `--from-indent` option
//! - `<delimiter>` : specified by `--from-delimiter` option
//! 
//! ### `dir_tree`
//! 
//! Directory tree with the glob pattern specified by `--from-glob-pattern` (default: `**/*`)
//! 
//! ### `html_list`
//! 
//! HTML `<ul><li>` and/or `<ol><li>` [nesting list](https://www.w3.org/wiki/HTML_lists#Nesting_lists).
//! All text outside of `<li>` elements is ignored.
//! 
//! ### `mspdi`
//! 
//! MS Project 20xx XML Data Interchange (i.e. files saved as "XML" format on MS Project).
//! Treat the task name as a key text, the other attributes as values.
//! 
//! ### `opml`
//! 
//! [OPML](http://dev.opml.org/)
//! Treat the `text` attribute as a key text, the other attributes as values.
//! 
//! ## Types of Output
//! 
//! The sample input used in this section are as follows:
//! 
//! ```text
//! 1,1(1),1(2)
//!   1.1,1.1(1),1.1(2)
//!   1.2,1.2(1),1.2(2)
//!     1.2.1,1.2.1(1),1.2.1(2)
//! ```
//! 
//! - key header: H1, H2, H3
//! - value header: H(1), H(2)
//! 
//! ### Common Options
//! 
//! `--shironuri=yes` : fill all the cells with white color
//! 
//! ### `xlsx_type0`
//! 
//! Basic XLSX output format.
//! 
//! ### `xlsx_type1`
//! 
//! XLSX output with row outlining.
//! 
//! #### Options for `xlsx_type1`
//! 
//! `--outline-rows=yes` : group rows
//! 
//! ### `xlsx_type2`
//! 
//! XLSX output with cell integration (colspan, rowspan).
//! 
//! #### Options for `xlsx_type2`
//! 
//! `--integrate-cells={colspan,rowspan}` : group columns/rows.
//! `--outline-rows=yes` : group rows.
//! 
//! ### `xlsx_type3`
//! 
//! Advanced XLSX output with specific header and item cell layouts, and cell integration (colspan, rowspan, both).
//! 
//! #### Options for `xlsx_type3`
//! 
//! `--integrate-cells={colspan,rowspan,both}` : group columns/rows.
//! 
//! ### `xlsx_type4`
//! 
//! XLSX output with cell integration (colspan, rowspan).
//! 
//! #### Options for `xlsx_type4`
//! 
//! `--integrate-cells={colspan,rowspan,both}` : group columns/rows.
//! 
//! ### `xlsx_type5`
//! 
//! XLSX output with cell integration (colspan, rowspan).
//! 
//! #### Options for `xlsx_type5`
//! 
//! `--integrate-cells=colspan` : group columns/rows.

pub mod cli;
pub mod docs;
pub mod generator;
pub mod outline;
pub mod parser;

pub fn get_parser_types() -> Vec<String> {
    vec![
        "simple_text".to_string(),
        "dir_tree".to_string(),
        "html_list".to_string(),
        "mspdi".to_string(),
        "opml".to_string(),
    ]
}

pub fn get_generator_types() -> Vec<String> {
    vec![
        "xlsx_type0".to_string(),
        "xlsx_type1".to_string(),
        "xlsx_type2".to_string(),
        "xlsx_type3".to_string(),
        "xlsx_type4".to_string(),
        "xlsx_type5".to_string(),
    ]
}
