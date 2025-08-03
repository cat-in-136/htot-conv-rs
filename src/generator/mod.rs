pub mod base;
pub mod xlsx_type0;
pub mod xlsx_type1;

#[derive(Debug, Clone)]
pub enum GeneratorOptions {
    XlsxType0(xlsx_type0::XlsxType0GeneratorOptions),
    XlsxType1(xlsx_type1::XlsxType1GeneratorOptions),
}
