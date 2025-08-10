use crate::generator::base::IntegrateCellsOption;
use crate::outline::{Outline, OutlineTree};
use anyhow::Result;
use clap::Args;
use rust_xlsxwriter::{Format, FormatBorder, Worksheet};
use std::rc::Rc;

#[derive(Debug, Clone, Args)]
pub struct XlsxType5GeneratorOptions {
    /// integrate key cells (specify 'colspan')
    #[arg(long)]
    pub integrate_cells: Option<IntegrateCellsOption>,
}

pub struct XlsxType5Generator {
    outline: Outline,
    options: XlsxType5GeneratorOptions,
}

impl XlsxType5Generator {
    pub fn new(outline: Outline, options: XlsxType5GeneratorOptions) -> Self {
        XlsxType5Generator { outline, options }
    }

    pub fn output_to_worksheet(&self, worksheet: &mut Worksheet) -> Result<()> {
        let max_level = self.outline.max_level() as usize;
        let max_value_length = self.outline.max_value_length();

        let header_format = Format::new().set_border(FormatBorder::Thin);
        let item_format = Format::new().set_border(FormatBorder::Thin); // Used for merged cells

        // Write header row
        let mut col_index = 0;
        for level in 1..=max_level {
            let header_text = self
                .outline
                .key_header
                .get(level - 1)
                .map_or("".to_string(), |s| s.clone());
            worksheet.write_string_with_format(
                0, // row_index
                col_index as u16,
                &header_text,
                &header_format,
            )?;
            col_index += 1;
        }

        for i in 0..max_value_length {
            let header_text = self
                .outline
                .value_header
                .get(i)
                .map_or("".to_string(), |s| s.clone());
            worksheet.write_string_with_format(
                0, // row_index
                col_index as u16,
                &header_text,
                &header_format,
            )?;
            col_index += 1;
        }

        let mut row_index: u32 = 1; // Start from row 1 for data

        let tree = self.outline.to_tree();
        for node_rc in OutlineTree::descendants(&tree) {
            let node = node_rc.borrow();
            if node.is_leaf() {
                let item = node.item().unwrap(); // Leaf node must have an item

                let mut key_cell: Vec<Option<String>> = vec![None; max_level];
                key_cell[item.level as usize - 1] = Some(item.key.clone());

                // Populate ancestors' keys (parent key repetition)
                let mut current_node_rc = Rc::clone(&node_rc);
                loop {
                    let parent_option = {
                        let node_borrowed = current_node_rc.borrow();
                        node_borrowed.parent().map(|p| Rc::clone(&p))
                    };

                    if let Some(parent_rc) = parent_option {
                        let parent_node = parent_rc.borrow();
                        if let Some(parent_item) = parent_node.item() {
                            key_cell[parent_item.level as usize - 1] =
                                Some(parent_item.key.clone());
                        }
                        current_node_rc = Rc::clone(&parent_rc);
                    } else {
                        break;
                    }
                }

                let value_cell = item
                    .value
                    .iter()
                    .map(|s| Some(s.clone()))
                    .collect::<Vec<Option<String>>>();
                let padded_value_cell = pad_array(value_cell, max_value_length);

                let combined_cells: Vec<Option<String>> = key_cell
                    .into_iter()
                    .chain(padded_value_cell.into_iter())
                    .collect();

                for (c_idx, cell_val_opt) in combined_cells.iter().enumerate() {
                    let cell_val = cell_val_opt.as_deref().unwrap_or(""); // Get string slice or empty string
                    worksheet.write_string_with_format(
                        row_index,
                        c_idx as u16,
                        cell_val,
                        &item_format, // Always apply thin border
                    )?;
                }

                // Border logic from Ruby XlsxType5
                for level in item.level as usize..=max_level {
                    let mut has_border = false;

                    if level != item.level as usize {
                        // unless (level == item.level)
                        has_border = true;
                    }
                    if level != max_level {
                        // unless (level == max_level)
                        has_border = true;
                    }

                    if has_border {
                        let mut current_format = item_format.clone();
                        if level != item.level as usize {
                            current_format = current_format.set_border_left(FormatBorder::None);
                        }
                        if level != max_level {
                            current_format = current_format.set_border_right(FormatBorder::None);
                        }
                        worksheet.write_string_with_format(
                            row_index,
                            (level - 1) as u16,
                            combined_cells[level - 1].clone().unwrap_or_default(),
                            &current_format,
                        )?;
                    }
                }

                // Colspan merging
                if self.options.integrate_cells == Some(IntegrateCellsOption::Colspan) && item.level < max_level as u32 {
                    let first_col = item.level as u16 - 1;
                    let last_col = max_level as u16 - 1;
                    worksheet.merge_range(
                        row_index,
                        first_col,
                        row_index,
                        last_col,
                        &item.key,    // The key of the current item
                        &item_format, // Use item_format for merged cell
                    )?;
                }

                row_index += 1;
            }
        }

        Ok(())
    }
}

// Helper function for padding array, similar to Ruby's Util.pad_array
fn pad_array(mut arr: Vec<Option<String>>, target_len: usize) -> Vec<Option<String>> {
    while arr.len() < target_len {
        arr.push(None);
    }
    arr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outline::Outline;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;
    use umya_spreadsheet::reader::xlsx::read as read_xlsx;

    // Helper to create a reference outline similar to Ruby's reference_outline
    fn create_reference_outline() -> Outline {
        let mut outline = Outline::new();
        outline.key_header = vec!["H1".into(), "H2".into(), "H3".into()];
        outline.value_header = vec!["H(1)".into(), "H(2)".into()];
        outline.add_item("1", 1, vec![]);
        outline.add_item("1.1", 2, vec!["1.1(1)".into(), "1.1(2)".into()]);
        outline.add_item("1.2", 2, vec![]);
        outline.add_item("1.2.1", 3, vec!["1.2.1(1)".into(), "1.2.1(2)".into()]);
        outline
    }

    #[test]
    fn test_output_worksheet_basic() -> Result<()> {
        let outline = create_reference_outline();
        let gen = XlsxType5Generator::new(
            outline.clone(),
            XlsxType5GeneratorOptions {
                integrate_cells: None,
            },
        );

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        gen.output_to_worksheet(worksheet)?;

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let ws = read_spreadsheet.get_sheet(&0).unwrap();

        // Header row values (umya_spreadsheet is 1-based for rows and columns)
        assert_eq!(ws.get_value((1, 1)).as_str(), "H1");
        assert_eq!(ws.get_value((2, 1)).as_str(), "H2");
        assert_eq!(ws.get_value((3, 1)).as_str(), "H3");
        assert_eq!(ws.get_value((4, 1)).as_str(), "H(1)");
        assert_eq!(ws.get_value((5, 1)).as_str(), "H(2)");

        // Data row 1 (Ruby expected: ["1", "1.1", nil, "1.1(1)", "1.1(2)"])
        // XlsxType5 repeats parent keys, so A2 should be "1"
        assert_eq!(ws.get_value((1, 2)).as_str(), "1"); // A2
        assert_eq!(ws.get_value((2, 2)).as_str(), "1.1"); // B2
        assert_eq!(ws.get_value((3, 2)).as_str(), ""); // C2 (nil in Ruby)
        assert_eq!(ws.get_value((4, 2)).as_str(), "1.1(1)"); // D2
        assert_eq!(ws.get_value((5, 2)).as_str(), "1.1(2)"); // E2

        // Data row 2 (Ruby expected: ["1", "1.2", "1.2.1", "1.2.1(1)", "1.2.1(2)"])
        // XlsxType5 repeats parent keys, so A3 should be "1"
        assert_eq!(ws.get_value((1, 3)).as_str(), "1"); // A3
        assert_eq!(ws.get_value((2, 3)).as_str(), "1.2"); // B3
        assert_eq!(ws.get_value((3, 3)).as_str(), "1.2.1"); // C3
        assert_eq!(ws.get_value((4, 3)).as_str(), "1.2.1(1)"); // D3
        assert_eq!(ws.get_value((5, 3)).as_str(), "1.2.1(2)"); // E3

        // No merged cells without integrate_cells option
        assert_eq!(ws.get_merge_cells().len(), 0);

        // Auto-filter check: Removed as per user request
        // assert!(ws.get_auto_filter().is_some());
        // let auto_filter = ws.get_auto_filter().unwrap();
        // umya_spreadsheet's get_range() returns A1:E3 for 1-based (1,1) to (5,3)
        // assert_eq!(auto_filter.get_range(), "A1:E3");

        Ok(())
    }

    #[test]
    fn test_output_worksheet_with_integrate_cells_colspan() -> Result<()> {
        let outline = create_reference_outline();
        let gen = XlsxType5Generator::new(
            outline.clone(),
            XlsxType5GeneratorOptions {
                integrate_cells: Some(IntegrateCellsOption::Colspan),
            },
        );

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        gen.output_to_worksheet(worksheet)?;

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let ws = read_spreadsheet.get_sheet(&0).unwrap();

        // Check merged cells
        let merged_cells: Vec<String> =
            ws.get_merge_cells().iter().map(|m| m.get_range()).collect();
        assert_eq!(merged_cells, vec!["B2:C2".to_string()]); // Ruby test expects B2:C2

        Ok(())
    }
}
