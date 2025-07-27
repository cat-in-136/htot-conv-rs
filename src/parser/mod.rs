pub mod dir_tree;
pub mod simple_text;

pub enum ParserOptions {
    SimpleText(simple_text::SimpleTextParserOptions),
    DirTree(dir_tree::DirTreeParserOptions),
}
