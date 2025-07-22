use crate::outline::Outline;
use anyhow::Result;
use rust_xlsxwriter::Worksheet;

pub trait Generator {
    fn output_to_worksheet(&self, worksheet: &mut Worksheet, data: &Outline) -> Result<()>;
}
