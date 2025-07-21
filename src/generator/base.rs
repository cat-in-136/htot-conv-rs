use crate::outline::Outline;
use anyhow::Result;
use umya_spreadsheet::Worksheet;

pub trait Generator {
    fn output_to_worksheet(&self, ws: &mut Worksheet, data: &Outline) -> Result<()>;
}
