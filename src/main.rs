use clap::Parser;
use htot_conv_rs::cli::run_conversion;

use htot_conv_rs::generator::xlsx_type0::XlsxType0GeneratorOptions;
use htot_conv_rs::generator::xlsx_type1::XlsxType1GeneratorOptions;
use htot_conv_rs::generator::xlsx_type2::XlsxType2GeneratorOptions;
use htot_conv_rs::generator::xlsx_type3::XlsxType3GeneratorOptions;
use htot_conv_rs::generator::xlsx_type4::XlsxType4GeneratorOptions;
use htot_conv_rs::generator::xlsx_type5::XlsxType5GeneratorOptions;
use htot_conv_rs::generator::GeneratorOptions;
use htot_conv_rs::parser::dir_tree::DirTreeParserOptions;
use htot_conv_rs::parser::html_list::HtmlListParserOptions;
use htot_conv_rs::parser::mspdi::MspdiParserOptions;
use htot_conv_rs::parser::opml::OpmlParserOptions;
use htot_conv_rs::parser::simple_text::SimpleTextParserOptions;
use htot_conv_rs::parser::ParserOptions;
use htot_conv_rs::{get_generator_types, get_parser_types};

use std::fs::File;
use std::io::{self, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Type of input (e.g., simple_text, dir_tree)
    #[arg(short = 'f', long, value_name = "TYPE", default_value = "simple_text")]
    from_type: String,

    /// Type of output (e.g., xlsx_type0, xlsx_type1)
    #[arg(short = 't', long, value_name = "TYPE", default_value = "xlsx_type2")]
    to_type: String,

    /// The string used for indentation (e.g., "  " for two spaces, "\t" for tab).
    #[arg(long = "from-indent", default_value = "\t")]
    indent: String,
    /// An optional delimiter string used to separate the key from its values.
    #[arg(long = "from-delimiter")]
    delimiter: Option<String>,
    /// If true, empty lines in the input will be preserved as level-1 items.
    #[arg(long = "from-preserve-empty-line")]
    preserve_empty_line: bool,
    /// A comma-separated list of strings representing the key headers (for simple_text, dir_tree, html_list, mspdi, opml).
    #[arg(long = "from-key-header")]
    key_header: Option<String>,
    /// A comma-separated list of strings representing the value headers (for simple_text, mspdi, opml).
    #[arg(long = "from-value-header")]
    value_header: Option<String>,

    /// Glob pattern for dir_tree parser (e.g., "**/*", "*.txt").
    #[arg(long = "from-glob-pattern", default_value = "**/*")]
    glob_pattern: Option<String>,
    /// Directory indicator for dir_tree parser (e.g., "/").
    #[arg(long = "from-dir-indicator")]
    dir_indicator: Option<String>,

    /// Group rows in XLSX output (for xlsx_type1, xlsx_type2, xlsx_type3).
    #[arg(long = "to-outline-rows", default_value_t = false)]
    to_outline_rows: bool,

    /// Integrate cells in XLSX output (for xlsx_type2, xlsx_type3, xlsx_type4, xlsx_type5).
    #[arg(long = "to-integrate-cells")]
    to_integrate_cells: Option<htot_conv_rs::generator::base::IntegrateCellsOption>,

    /// Fill all cells with white color.
    #[arg(long = "to-shironuri", default_value_t = false)]
    to_shironuri: bool,

    /// Input file (default: stdin)
    input: Option<String>,

    /// Output file (default: stdout)
    output: Option<String>,

    /// List available input/output types
    #[arg(short = 'l', long)]
    list_type: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.list_type {
        println!("type of input:");
        println!("{}", get_parser_types().join(" "));
        println!();
        println!("type of output:");
        println!("{}", get_generator_types().join(" "));
        println!();
        return Ok(());
    }

    let input_path_option = cli.input;
    let output_path = cli.output;

    // Prepare output writer
    let mut output_writer: Box<dyn Write> = match output_path {
        Some(path) if path != "-" => Box::new(File::create(path)?),
        _ => Box::new(io::stdout()),
    };

    let parsed_key_header: Vec<String> = cli
        .key_header
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    let parsed_value_header: Vec<String> = cli
        .value_header
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let from_options = match cli.from_type.as_str() {
        "simple_text" => ParserOptions::SimpleText(SimpleTextParserOptions {
            indent: cli.indent,
            delimiter: cli.delimiter,
            preserve_empty_line: cli.preserve_empty_line,
            key_header: parsed_key_header,
            value_header: parsed_value_header,
        }),
        "dir_tree" => ParserOptions::DirTree(DirTreeParserOptions {
            key_header: parsed_key_header,
            glob_pattern: cli.glob_pattern,
            dir_indicator: cli.dir_indicator,
        }),
        "html_list" => ParserOptions::HtmlList(HtmlListParserOptions {
            key_header: parsed_key_header,
        }),
        "mspdi" => ParserOptions::Mspdi(MspdiParserOptions {
            key_header: parsed_key_header,
            value_header: parsed_value_header,
        }),
        "opml" => ParserOptions::Opml(OpmlParserOptions {
            key_header: parsed_key_header,
            value_header: parsed_value_header,
        }),
        _ => anyhow::bail!(
            "Unsupported from_type: {}. Supported types are: {}",
            cli.from_type,
            get_parser_types().join(", ")
        ),
    };

    let to_options = match cli.to_type.as_str() {
        "xlsx_type0" => GeneratorOptions::XlsxType0(XlsxType0GeneratorOptions {
            shironuri: cli.to_shironuri,
        }),
        "xlsx_type1" => GeneratorOptions::XlsxType1(XlsxType1GeneratorOptions {
            outline_rows: cli.to_outline_rows,
            shironuri: cli.to_shironuri,
        }),
        "xlsx_type2" => GeneratorOptions::XlsxType2(XlsxType2GeneratorOptions {
            outline_rows: cli.to_outline_rows,
            integrate_cells: cli.to_integrate_cells,
            shironuri: cli.to_shironuri,
        }),
        "xlsx_type3" => GeneratorOptions::XlsxType3(XlsxType3GeneratorOptions {
            outline_rows: cli.to_outline_rows,
            integrate_cells: cli.to_integrate_cells,
            shironuri: cli.to_shironuri,
        }),
        "xlsx_type4" => GeneratorOptions::XlsxType4(XlsxType4GeneratorOptions {
            integrate_cells: cli.to_integrate_cells,
            shironuri: cli.to_shironuri,
        }),
        "xlsx_type5" => GeneratorOptions::XlsxType5(XlsxType5GeneratorOptions {
            integrate_cells: cli.to_integrate_cells,
            shironuri: cli.to_shironuri,
        }),
        _ => anyhow::bail!(
            "Unsupported to_type: {}. Supported types are: {}",
            cli.to_type,
            get_generator_types().join(", ")
        ),
    };

    run_conversion(
        &input_path_option,
        &mut output_writer,
        from_options,
        to_options,
    )?;

    Ok(())
}
