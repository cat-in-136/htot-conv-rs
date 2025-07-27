pub mod base;
pub mod xlsx_type0;

#[derive(Debug, Clone)]
pub enum GeneratorOptions {
    XlsxType0(xlsx_type0::XlsxType0GeneratorOptions),
}
