//! Detailed documentation for input and output types.
//! 
//! This module contains comprehensive documentation about the various
//! input parsers and output generators supported by htot-conv-rs.

/// # Types of Input
/// 
/// ## `simple_text`
/// 
/// A text file consisting of multiple lines where:
/// - `<line> ::= { <indent> } <key> { <delimiter> <value> }`
/// - `<key>` : a text that does not start with `<indent>` and does not contain `<delimiter>` (if `<delimiter>` specified).
/// - `<value>` : a text that does not contain `<delimiter>`.
/// - `<indent>` : specified by `--from-indent` option
/// - `<delimiter>` : specified by `--from-delimiter` option
/// 
/// ## `dir_tree`
/// 
/// Directory tree with the glob pattern specified by `--from-glob-pattern` (default: `**/*`)
/// 
/// ## `html_list`
/// 
/// HTML `<ul><li>` and/or `<ol><li>` [nesting list](https://www.w3.org/wiki/HTML_lists#Nesting_lists).
/// All text outside of `<li>` elements is ignored.
/// 
/// ## `mspdi`
/// 
/// MS Project 20xx XML Data Interchange (i.e. files saved as "XML" format on MS Project).
/// Treat the task name as a key text, the other attributes as values.
/// 
/// ## `opml`
/// 
/// [OPML](http://dev.opml.org/)
/// Treat the `text` attribute as a key text, the other attributes as values.
pub mod input_types {
    /// Documentation for simple_text input format
    pub mod simple_text {
        //! Simple text format with indentation-based hierarchy
    }
    
    /// Documentation for dir_tree input format
    pub mod dir_tree {
        //! Directory tree structure parser
    }
    
    /// Documentation for html_list input format
    pub mod html_list {
        //! HTML list structure parser
    }
    
    /// Documentation for mspdi input format
    pub mod mspdi {
        //! Microsoft Project XML parser
    }
    
    /// Documentation for opml input format
    pub mod opml {
        //! OPML (Outline Processor Markup Language) parser
    }
}

/// # Types of Output
/// 
/// The sample input used in this section are as follows:
/// 
/// ```text
/// 1,1(1),1(2)
///   1.1,1.1(1),1.1(2)
///   1.2,1.2(1),1.2(2)
///     1.2.1,1.2.1(1),1.2.1(2)
/// ```
/// 
/// - key header: H1, H2, H3
/// - value header: H(1), H(2)
/// 
/// ## Common Options
/// 
/// `--shironuri=yes` : fill all the cells with white color
/// 
/// ## `xlsx_type0`
/// 
/// Basic XLSX output format.
/// 
/// ## `xlsx_type1`
/// 
/// XLSX output with row outlining.
/// 
/// ### Options for `xlsx_type1`
/// 
/// `--outline-rows=yes` : group rows
/// 
/// ## `xlsx_type2`
/// 
/// XLSX output with cell integration (colspan, rowspan).
/// 
/// ### Options for `xlsx_type2`
/// 
/// `--integrate-cells={colspan,rowspan}` : group columns/rows.
/// `--outline-rows=yes` : group rows.
/// 
/// ## `xlsx_type3`
/// 
/// Advanced XLSX output with headers and cell integration (colspan, rowspan, both).
/// 
/// ### Options for `xlsx_type3`
/// 
/// `--integrate-cells={colspan,rowspan,both}` : group columns/rows.
/// 
/// ## `xlsx_type4`
/// 
/// XLSX output with cell integration (colspan, rowspan).
/// 
/// ### Options for `xlsx_type4`
/// 
/// `--integrate-cells={colspan,rowspan,both}` : group columns/rows.
/// 
/// ## `xlsx_type5`
/// 
/// XLSX output with cell integration (colspan, rowspan).
/// 
/// ### Options for `xlsx_type5`
/// 
/// `--integrate-cells=colspan` : group columns/rows.
pub mod output_types {
    /// Documentation for xlsx_type0 output format
    pub mod xlsx_type0 {
        //! Basic XLSX output format
    }
    
    /// Documentation for xlsx_type1 output format
    pub mod xlsx_type1 {
        //! XLSX output with row outlining
    }
    
    /// Documentation for xlsx_type2 output format
    pub mod xlsx_type2 {
        //! XLSX output with cell integration
    }
    
    /// Documentation for xlsx_type3 output format
    pub mod xlsx_type3 {
        //! Advanced XLSX output with headers
    }
    
    /// Documentation for xlsx_type4 output format
    pub mod xlsx_type4 {
        //! XLSX output with cell integration
    }
    
    /// Documentation for xlsx_type5 output format
    pub mod xlsx_type5 {
        //! XLSX output with cell integration
    }
}