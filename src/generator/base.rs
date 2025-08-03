use crate::outline::Outline;
use anyhow::Result;
use clap::ValueEnum;
use rust_xlsxwriter::Worksheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IntegrateCellsOption {
    Colspan,
    Rowspan,
    Both,
}

pub trait Generator {
    fn output_to_worksheet(&self, worksheet: &mut Worksheet, data: &Outline) -> Result<()>;
}
