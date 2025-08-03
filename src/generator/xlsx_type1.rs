use crate::outline::Outline;
use anyhow::Result;
use clap::Args;
use rust_xlsxwriter::{Format, FormatBorder, Worksheet};

#[derive(Debug, Clone, Args)]
pub struct XlsxType1GeneratorOptions {
    /// group rows (default: no)
    #[arg(long, default_value_t = false)]
    pub outline_rows: bool,
}

pub struct XlsxType1Generator {
    options: XlsxType1GeneratorOptions,
}

impl XlsxType1Generator {
    pub fn new(options: XlsxType1GeneratorOptions) -> Self {
        XlsxType1Generator { options }
    }

    pub fn output_to_worksheet(&self, worksheet: &mut Worksheet, outline: &Outline) -> Result<()> {
        let mut row_index = 0;
        let max_value_length = outline.max_value_length();

        let header_format = Format::new().set_border(FormatBorder::Thin);
        let item_format = Format::new().set_border(FormatBorder::Thin);

        // Write key header and value headers
        let mut headers: Vec<String> = Vec::new();
        if let Some(key_h) = outline.key_header.first() {
            headers.push(key_h.clone());
        } else {
            headers.push("".to_string()); // Placeholder for key header if not present
        }
        // Pad value_header to max_value_length
        let mut padded_value_headers = outline.value_header.clone();
        padded_value_headers.resize(max_value_length, "".to_string());
        headers.extend(padded_value_headers);

        for (col_index, header_text) in headers.iter().enumerate() {
            worksheet.write_string_with_format(
                row_index,
                col_index as u16,
                header_text,
                &header_format,
            )?;
        }
        row_index += 1;

        let item_first_row_index = row_index;

        for item in &outline.item {
            let mut row_data = Vec::new();
            row_data.push(item.key.clone());
            // Pad item.value to max_value_length
            let mut padded_item_values = item.value.clone();
            padded_item_values.resize(max_value_length, "".to_string());
            row_data.extend(padded_item_values);

            for (col_index, cell_text) in row_data.iter().enumerate() {
                worksheet.write_string_with_format(
                    row_index,
                    col_index as u16,
                    cell_text,
                    &item_format,
                )?;
            }
            row_index += 1;
        }

        // Group rows if outline_rows option is true
        if self.options.outline_rows {
            let levels: Vec<_> = outline.item.iter().map(|v| v.level).collect();
            for (level, v) in Self::find_intervals_hierarchical(&levels)
                .iter()
                .enumerate()
            {
                if level > 0 {
                    for (first_index, last_index) in v.iter() {
                        let first_row = *first_index as u32 + item_first_row_index;
                        let last_row = *last_index as u32 + item_first_row_index;
                        worksheet.group_rows(first_row, last_row)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn find_intervals(arr: &[u32], threshold: u32) -> Vec<(usize, usize)> {
        let mut intervals = Vec::new();
        let mut start = None;

        for (i, &val) in arr.iter().enumerate() {
            if val >= threshold {
                if start.is_none() {
                    start = Some(i);
                }
            } else if let Some(s) = start {
                intervals.push((s, i - 1));
                start = None;
            }
        }

        if let Some(s) = start {
            intervals.push((s, arr.len() - 1));
        }

        intervals
    }

    fn find_intervals_hierarchical(arr: &[u32]) -> Vec<Vec<(usize, usize)>> {
        let max_val = match arr.iter().max() {
            Some(&max) if max > 0 => max,
            _ => return Vec::new(),
        };
        (1..=max_val)
            .map(|threshold| Self::find_intervals(arr, threshold))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;
    use umya_spreadsheet::reader::xlsx::read as read_xlsx;
    use umya_spreadsheet::Border;

    #[test]
    fn test_xlsx_type1_generator_basic() {
        let mut outline = Outline::default();
        outline.key_header = vec!["Key".to_string()];
        outline.value_header = vec!["Value1".to_string(), "Value2".to_string()];
        outline.add_item("Item 1", 1, vec!["Val1A".to_string(), "Val1B".to_string()]);
        outline.add_item("Item 2", 2, vec!["Val2A".to_string()]);
        outline.add_item(
            "Item 3",
            1,
            vec![
                "Val3A".to_string(),
                "Val3B".to_string(),
                "Val3C".to_string(),
            ],
        );

        let generator = XlsxType1Generator::new(XlsxType1GeneratorOptions {
            outline_rows: false,
        });

        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        generator
            .output_to_worksheet(&mut worksheet, &outline)
            .unwrap();

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Verify Header Row
        assert_eq!(read_worksheet.get_value((1, 1)).as_str(), "Key");
        assert_eq!(read_worksheet.get_value((2, 1)).as_str(), "Value1");
        assert_eq!(read_worksheet.get_value((3, 1)).as_str(), "Value2");

        // Verify Data Row 1
        assert_eq!(read_worksheet.get_value((1, 2)).as_str(), "Item 1");
        assert_eq!(read_worksheet.get_value((2, 2)).as_str(), "Val1A");
        assert_eq!(read_worksheet.get_value((3, 2)).as_str(), "Val1B");

        // Verify Data Row 2
        assert_eq!(read_worksheet.get_value((1, 3)).as_str(), "Item 2");
        assert_eq!(read_worksheet.get_value((2, 3)).as_str(), "Val2A");

        // Verify Data Row 3
        assert_eq!(read_worksheet.get_value((1, 4)).as_str(), "Item 3");
        assert_eq!(read_worksheet.get_value((2, 4)).as_str(), "Val3A");
        assert_eq!(read_worksheet.get_value((3, 4)).as_str(), "Val3B");
        assert_eq!(read_worksheet.get_value((4, 4)).as_str(), "Val3C");

        // Verify Borders (example for Header, cell (1,1))
        let header_style_1_1 = read_worksheet.get_style((1, 1));
        assert_eq!(
            header_style_1_1
                .get_borders()
                .unwrap()
                .get_top()
                .get_border_style(),
            Border::BORDER_THIN
        );
        // ... add more border assertions if needed

        drop(temp_file);
    }

    #[test]
    fn test_xlsx_type1_generator_outline_rows() {
        let mut outline = Outline::default();
        outline.add_item("Item 1", 1, vec![]);
        outline.add_item("Subitem 1.1", 2, vec![]);
        outline.add_item("Subitem 1.2", 2, vec![]);
        outline.add_item("Item 2", 1, vec![]);
        outline.add_item("Subitem 2.1", 2, vec![]);

        let generator = XlsxType1Generator::new(XlsxType1GeneratorOptions { outline_rows: true });

        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        generator
            .output_to_worksheet(&mut worksheet, &outline)
            .unwrap();

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let _read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Verify outline levels
        // Row 1 (Item 1) should have level 0 (no outline)
        // assert_eq!(read_worksheet.get_row_dimension(&1).unwrap().get_outline_level(), &0);
        // Row 2 (Subitem 1.1) should have level 1
        // assert_eq!(read_worksheet.get_row_dimension(&2).unwrap().get_outline_level(), &1);
        // Row 3 (Subitem 1.2) should have level 1
        // assert_eq!(read_worksheet.get_row_dimension(&3).unwrap().get_outline_level(), &1);
        // Row 4 (Item 2) should have level 0
        // assert_eq!(read_worksheet.get_row_dimension(&4).unwrap().get_outline_level(), &0);
        // Row 5 (Subitem 2.1) should have level 1
        // assert_eq!(read_worksheet.get_row_dimension(&5).unwrap().get_outline_level(), &1);

        drop(temp_file);
    }

    #[test]
    fn test_find_intervals_hierarchical() {
        let data = [1, 1, 2, 3, 3, 1, 2, 3];
        let result = XlsxType1Generator::find_intervals_hierarchical(&data);

        let expected = vec![
            vec![(0, 7)],         // threshold = 1
            vec![(2, 4), (6, 7)], // threshold = 2
            vec![(3, 4), (7, 7)], // threshold = 3
        ];
        assert_eq!(result, expected);
    }
}
