use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::Write;

use crate::generator::base::Generator;
use crate::generator::xlsx_type0::XlsxType0Generator;
use crate::generator::xlsx_type0::XlsxType0GeneratorOptions;
use crate::parser::simple_text::{SimpleTextParser, SimpleTextParserOptions};
use rust_xlsxwriter::Workbook;

#[allow(clippy::too_many_arguments)]
pub fn run_conversion(
    input_content: &str,
    output_writer: &mut dyn Write,
    from_type: &str,
    to_type: &str,
    _from_options: &HashMap<String, String>,
    _to_options: &HashMap<String, String>, // Prefix with _ to ignore unused warning
    simple_text_options: Option<SimpleTextParserOptions>,
    xlsx_type0_options: Option<XlsxType0GeneratorOptions>,
) -> Result<()> {
    let outline = match from_type {
        "simple_text" => {
            let parser = SimpleTextParser::new(simple_text_options.unwrap_or_default());
            parser.parse(input_content)?
        }
        _ => return Err(anyhow!("Unsupported parser type: {}", from_type)),
    };

    match to_type {
        "xlsx_type0" => {
            let generator = XlsxType0Generator::new(xlsx_type0_options.unwrap_or_default());
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            generator.output_to_worksheet(worksheet, &outline)?;

            // Save the workbook to a buffer and then write to the output_writer
            let buffer = workbook.save_to_buffer()?;
            output_writer.write_all(&buffer)?;
        }
        _ => return Err(anyhow!("Unsupported generator type: {}", to_type)),
    }

    Ok(())
}
