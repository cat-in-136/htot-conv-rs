use anyhow::Result;
use std::io::{Read, Write};

use crate::generator::base::Generator;
use crate::generator::xlsx_type0::XlsxType0Generator;
use crate::generator::xlsx_type1::XlsxType1Generator;
use crate::generator::xlsx_type2::XlsxType2Generator;
use crate::generator::xlsx_type3::XlsxType3Generator;
use crate::generator::GeneratorOptions;
use crate::parser::dir_tree::DirTreeParser;
use crate::parser::html_list::HtmlListParser;
use crate::parser::mspdi::MspdiParser;
use crate::parser::opml::OpmlParser;
use crate::parser::simple_text::SimpleTextParser;
use crate::parser::ParserOptions;
use rust_xlsxwriter::Workbook;

pub fn run_conversion(
    input_path_option: &Option<String>,
    output_writer: &mut dyn Write,
    from_options: ParserOptions,
    to_options: GeneratorOptions,
) -> Result<()> {
    let outline = match from_options {
        ParserOptions::SimpleText(options) => {
            let input_content = match input_path_option {
                Some(path) if path != "-" => std::fs::read_to_string(path)?,
                _ => {
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let parser = SimpleTextParser::new(options);
            parser.parse(&input_content)?
        }
        ParserOptions::DirTree(options) => {
            let path = match input_path_option {
                Some(p) => std::path::PathBuf::from(p),
                None => anyhow::bail!("Input path is required for dir_tree parser."),
            };
            if !path.is_dir() {
                anyhow::bail!(
                    "Input path '{}' is not a valid directory for dir_tree parser.",
                    path.display()
                );
            }
            let parser = DirTreeParser::new(options);
            parser.parse(&path)?
        }
        ParserOptions::HtmlList(options) => {
            let input_content = match input_path_option {
                Some(path) if path != "-" => std::fs::read_to_string(path)?,
                _ => {
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let parser = HtmlListParser::new(options);
            parser.parse(&input_content)?
        }
        ParserOptions::Mspdi(options) => {
            let input_content = match input_path_option {
                Some(path) if path != "-" => std::fs::read_to_string(path)?,
                _ => {
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let parser = MspdiParser::new(options);
            parser.parse(&input_content)?
        }
        ParserOptions::Opml(options) => {
            let input_content = match input_path_option {
                Some(path) if path != "-" => std::fs::read_to_string(path)?,
                _ => {
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let parser = OpmlParser::new(options);
            parser.parse(&input_content)?
        }
    };

    match to_options {
        GeneratorOptions::XlsxType0(options) => {
            let generator = XlsxType0Generator::new(options);
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            generator.output_to_worksheet(worksheet, &outline)?;

            // Save the workbook to a buffer and then write to the output_writer
            let buffer = workbook.save_to_buffer()?;
            output_writer.write_all(&buffer)?;
        }
        GeneratorOptions::XlsxType1(options) => {
            let generator = XlsxType1Generator::new(options);
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            generator.output_to_worksheet(worksheet, &outline)?;

            // Save the workbook to a buffer and then write to the output_writer
            let buffer = workbook.save_to_buffer()?;
            output_writer.write_all(&buffer)?;
        }
        GeneratorOptions::XlsxType2(options) => {
            let generator = XlsxType2Generator::new(outline, options);
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            generator.output_to_worksheet(worksheet)?;

            // Save the workbook to a buffer and then write to the output_writer
            let buffer = workbook.save_to_buffer()?;
            output_writer.write_all(&buffer)?;
        }
        GeneratorOptions::XlsxType3(options) => {
            let generator = XlsxType3Generator::new(outline, options);
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            generator.output_to_worksheet(worksheet)?;

            // Save the workbook to a buffer and then write to the output_writer
            let buffer = workbook.save_to_buffer()?;
            output_writer.write_all(&buffer)?;
        }
    };

    Ok(())
}
