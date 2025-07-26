use clap::Parser;
use htot_conv_rs::cli::run_conversion;
use htot_conv_rs::generator::xlsx_type0::XlsxType0GeneratorOptions;
use htot_conv_rs::parser::simple_text::SimpleTextParserOptions;
use htot_conv_rs::{get_generator_types, get_parser_types};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Type of input (e.g., simple_text, dir_tree)
    #[arg(short = 'f', long, value_name = "TYPE", default_value = "simple_text")]
    from_type: String,

    /// Type of output (e.g., xlsx_type0, xlsx_type1)
    #[arg(short = 't', long, value_name = "TYPE", default_value = "xlsx_type2")]
    to_type: String,

    /// Input file (default: stdin)
    input: Option<String>,

    /// Output file (default: stdout)
    output: Option<String>,

    #[clap(flatten)]
    simple_text_options: Option<SimpleTextParserOptions>,

    #[clap(flatten)]
    xlsx_type0_options: Option<XlsxType0GeneratorOptions>,

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

    let from_type = cli.from_type;
    let to_type = cli.to_type;
    let input_path = cli.input;
    let output_path = cli.output;

    let from_options: HashMap<String, String> = HashMap::new(); // Placeholder

    let to_options: HashMap<String, String> = HashMap::new(); // Placeholder

    // Read input
    let mut input_content = String::new();
    match input_path {
        Some(path) if path != "-" => {
            File::open(path)?.read_to_string(&mut input_content)?;
        }
        _ => {
            io::stdin().read_to_string(&mut input_content)?;
        }
    }

    // Prepare output writer
    let mut output_writer: Box<dyn Write> = match output_path {
        Some(path) if path != "-" => Box::new(File::create(path)?),
        _ => Box::new(io::stdout()),
    };

    run_conversion(
        &input_content,
        &mut output_writer,
        &from_type,
        &to_type,
        &from_options,
        &to_options,
        cli.simple_text_options,
        cli.xlsx_type0_options,
    )?;

    Ok(())
}
