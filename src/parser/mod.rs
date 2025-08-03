pub mod dir_tree;
pub mod html_list;
pub mod mspdi;
pub mod opml;
pub mod simple_text;

pub enum ParserOptions {
    SimpleText(simple_text::SimpleTextParserOptions),
    DirTree(dir_tree::DirTreeParserOptions),
    HtmlList(html_list::HtmlListParserOptions),
    Mspdi(mspdi::MspdiParserOptions),
    Opml(opml::OpmlParserOptions),
}
