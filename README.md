# htot-conv-rs - Hierarchical-Tree Outline Text Converter (Rust Port)

`htot-conv-rs` is a Rust re-implementation of the `htot_conv` Ruby project. The primary goals of this port are to enhance performance, ensure memory safety, and provide a more robust command-line interface (CLI) application. This project focuses solely on CLI functionality and does not aim to provide library features.

## Features

*   **Input Parsers**:
    *   `simple_text`: Parses hierarchical text outlines with indentation.
    *   `dir_tree`: Parses directory structures into outlines.
    *   `html_list`: Parses HTML list structures (`<ul>`, `<ol>`) into outlines.
    *   `mspdi`: Parses Microsoft Project XML (MSPDI) files into outlines.
    *   `opml`: Parses OPML (Outline Processor Markup Language) files into outlines.
*   **Output Generators (XLSX)**:
    *   `xlsx_type0`: Basic XLSX output.
    *   `xlsx_type1`: XLSX output with row outlining.
    *   `xlsx_type2`: XLSX output with cell integration (colspan, rowspan).
    *   `xlsx_type3`: Advanced XLSX output with specific header and item cell layouts, and cell integration (colspan, rowspan, both).

## Installation

To build and install `htot-conv-rs`, you need to have Rust and Cargo installed.

```bash
# Clone the repository
git clone https://github.com/your-repo/htot-conv-rs.git # Replace with actual repo URL
cd htot-conv-rs

# Build the project (release mode for optimized performance)
# The executable will be located at target/release/htot-conv-rs
```

## Usage

`htot-conv-rs` is a command-line tool. Here's an example of how to use it:

```bash
# Example: Convert a simple text outline to an XLSX file (xlsx_type3 with both cell integration)
# Assuming test_input.txt contains your outline data
# Example test_input.txt content:
# President
#   VP Marketing
#     Manager
#     Manager
#   VP Production
#     Manager
#     Manager
#   VP Sales
#     Manager
#     Manager

target/release/htot-conv-rs -f simple_text -t xlsx_type3 \
  --from-key-header Key --from-value-header Value1,Value2 \
  --from-indent="  " --from-delimiter=, \
  --to-integrate-cells=both \
  test_input.txt output.xlsx

# For more options and help:
target/release/htot-conv-rs --help
```

## Development

To set up the development environment and run tests:

```bash
# Clone the repository
git clone https://github.com/your-repo/htot-conv-rs.git # Replace with actual repo URL
cd htot-conv-rs

# Run tests
# cargo test

# Format code
# cargo fmt

# Check for linter warnings
# cargo clippy
```

## License

This project is licensed under the MIT License. See the `LICENSE.txt` file for details.
