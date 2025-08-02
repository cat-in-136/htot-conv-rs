pub mod dir_tree;
pub mod html_list;
pub mod simple_text;

pub enum ParserOptions {
    SimpleText(simple_text::SimpleTextParserOptions),
    DirTree(dir_tree::DirTreeParserOptions),
    HtmlList(html_list::HtmlListParserOptions),
}
