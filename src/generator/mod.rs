//! Output generators for different XLSX formats.
//! 
//! This module contains generators that can convert the internal outline
//! structure into various XLSX output formats.

pub mod xlsx_type0;
pub mod xlsx_type1;
pub mod xlsx_type2;
pub mod xlsx_type3;
pub mod xlsx_type4;
pub mod xlsx_type5;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IntegrateCellsOption {
    Colspan,
    Rowspan,
    Both,
}

#[derive(Debug, Clone)]
pub enum GeneratorOptions {
    XlsxType0(xlsx_type0::XlsxType0GeneratorOptions),
    XlsxType1(xlsx_type1::XlsxType1GeneratorOptions),
    XlsxType2(xlsx_type2::XlsxType2GeneratorOptions),
    XlsxType3(xlsx_type3::XlsxType3GeneratorOptions),
    XlsxType4(xlsx_type4::XlsxType4GeneratorOptions),
    XlsxType5(xlsx_type5::XlsxType5GeneratorOptions),
}
