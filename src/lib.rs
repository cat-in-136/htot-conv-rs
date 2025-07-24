pub mod cli;
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
