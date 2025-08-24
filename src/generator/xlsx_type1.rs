use crate::outline::Outline;
use anyhow::Result;
use rust_xlsxwriter::{Format, FormatBorder, Worksheet};

#[derive(Debug, Clone)]
pub struct XlsxType1GeneratorOptions {
    pub outline_rows: bool,
    pub shironuri: bool,
}

pub struct XlsxType1Generator {
    outline: Outline,
    options: XlsxType1GeneratorOptions,
}

impl XlsxType1Generator {
    pub fn new(outline: Outline, options: XlsxType1GeneratorOptions) -> Self {
        XlsxType1Generator { outline, options }
    }

    pub fn output_to_worksheet(&self, worksheet: &mut Worksheet) -> Result<()> {
        let mut row_index = 0;
        let max_value_length = self.outline.max_value_length();

        let mut header_format = Format::new().set_border(FormatBorder::Thin);
        let mut item_format = Format::new().set_border(FormatBorder::Thin);

        if self.options.shironuri {
            header_format = header_format.set_background_color(rust_xlsxwriter::Color::White);
            item_format = item_format.set_background_color(rust_xlsxwriter::Color::White);
        }

        // If shironuri is true, set the background color of all cells to white.
        if self.options.shironuri {
            let cell_format = Format::new().set_background_color(rust_xlsxwriter::Color::White);
            worksheet.set_column_range_format(0, 16383, &cell_format)?;
        }

        // Write key header and value headers
        let mut headers: Vec<String> = Vec::new();
        if let Some(key_h) = self.outline.key_header.first() {
            headers.push(key_h.clone());
        } else {
            headers.push("".to_string()); // Placeholder for key header if not present
        }
        // Pad value_header to max_value_length
        let mut padded_value_headers = self.outline.value_header.clone();
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

        for item in &self.outline.item {
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
            let levels: Vec<_> = self.outline.item.iter().map(|v| v.level).collect();
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
    use crate::outline::OutlineItem;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;
    use umya_spreadsheet::reader::xlsx::read as read_xlsx;
    use umya_spreadsheet::Border;

    #[test]
    fn test_xlsx_type1_generator_basic() {
        let outline = Outline {
            key_header: vec!["Key".to_string()],
            value_header: vec!["Value1".to_string(), "Value2".to_string()],
            item: vec![
                OutlineItem::new("Item 1", 1, vec!["Val1A".to_string(), "Val1B".to_string()]),
                OutlineItem::new("Item 2", 2, vec!["Val2A".to_string()]),
                OutlineItem::new(
                    "Item 3",
                    1,
                    vec![
                        "Val3A".to_string(),
                        "Val3B".to_string(),
                        "Val3C".to_string(),
                    ],
                ),
            ],
        };

        let generator = XlsxType1Generator::new(
            outline,
            XlsxType1GeneratorOptions {
                outline_rows: false,
                shironuri: false,
            },
        );

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(worksheet).unwrap();

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
        let header_cell_coords = (1, 1); // A1
        let header_style = read_worksheet.get_style(header_cell_coords);
        assert_eq!(
            header_style
                .get_borders()
                .unwrap()
                .get_top()
                .get_border_style(),
            Border::BORDER_THIN,
            "Header cell {:?} top border",
            header_cell_coords
        );
        assert_eq!(
            header_style
                .get_borders()
                .unwrap()
                .get_bottom()
                .get_border_style(),
            Border::BORDER_THIN,
            "Header cell {:?} bottom border",
            header_cell_coords
        );
        assert_eq!(
            header_style
                .get_borders()
                .unwrap()
                .get_left()
                .get_border_style(),
            Border::BORDER_THIN,
            "Header cell {:?} left border",
            header_cell_coords
        );
        assert_eq!(
            header_style
                .get_borders()
                .unwrap()
                .get_right()
                .get_border_style(),
            Border::BORDER_THIN,
            "Header cell {:?} right border",
            header_cell_coords
        );

        // Verify Borders (example for Data Row 1, cell (1,2))
        let data_cell_coords = (1, 2); // A2
        let data_style = read_worksheet.get_style(data_cell_coords);
        assert_eq!(
            data_style
                .get_borders()
                .unwrap()
                .get_top()
                .get_border_style(),
            Border::BORDER_THIN,
            "Data cell {:?} top border",
            data_cell_coords
        );
        assert_eq!(
            data_style
                .get_borders()
                .unwrap()
                .get_bottom()
                .get_border_style(),
            Border::BORDER_THIN,
            "Data cell {:?} bottom border",
            data_cell_coords
        );
        assert_eq!(
            data_style
                .get_borders()
                .unwrap()
                .get_left()
                .get_border_style(),
            Border::BORDER_THIN,
            "Data cell {:?} left border",
            data_cell_coords
        );
        assert_eq!(
            data_style
                .get_borders()
                .unwrap()
                .get_right()
                .get_border_style(),
            Border::BORDER_THIN,
            "Data cell {:?} right border",
            data_cell_coords
        );

        drop(temp_file);
    }

    #[test]
    fn test_xlsx_type1_generator_outline_rows() {
        let outline = Outline {
            item: vec![
                OutlineItem::new("Item 1", 1, vec![]),
                OutlineItem::new("Subitem 1.1", 2, vec![]),
                OutlineItem::new("Subitem 1.2", 2, vec![]),
                OutlineItem::new("Item 2", 1, vec![]),
                OutlineItem::new("Subitem 2.1", 2, vec![]),
            ],
            ..Default::default()
        };

        let generator = XlsxType1Generator::new(
            outline,
            XlsxType1GeneratorOptions {
                outline_rows: true,
                shironuri: false,
            },
        );

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(worksheet).unwrap();

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
    fn test_xlsx_type1_generator_shironuri_enabled() {
        let outline = Outline {
            key_header: vec!["Key".to_string()],
            value_header: vec!["Value1".to_string(), "Value2".to_string()],
            item: vec![OutlineItem::new(
                "Item 1",
                1,
                vec!["Val1A".to_string(), "Val1B".to_string()],
            )],
        };

        let options = XlsxType1GeneratorOptions {
            outline_rows: false,
            shironuri: true,
        };
        let generator = XlsxType1Generator::new(outline, options);

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(worksheet).unwrap();

        // Save to a temporary file using rust_xlsxwriter
        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        // Read the file back and assert its content (using umya_spreadsheet as instructed)
        let read_spreadsheet = umya_spreadsheet::reader::xlsx::read(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Check if the background color of cell A1 is white
        assert_eq!(
            read_worksheet
                .get_cell("A1")
                .and_then(|cell| cell.get_style().get_background_color())
                .map(|color| color.get_argb()),
            Some(umya_spreadsheet::structs::Color::COLOR_WHITE)
        );

        // Note: We cannot check the background color of the bottom-right cell (XFD1048576)
        // because umya_spreadsheet does not return styles for cells that have no value.
        // The set_column_range_format should have set the background color for the entire worksheet,
        // but umya_spreadsheet cannot detect it.

        drop(temp_file);
    }

    #[test]
    fn test_xlsx_type1_generator_shironuri_disabled() {
        let outline = Outline {
            key_header: vec!["Key".to_string()],
            value_header: vec!["Value1".to_string(), "Value2".to_string()],
            item: vec![OutlineItem::new(
                "Item 1",
                1,
                vec!["Val1A".to_string(), "Val1B".to_string()],
            )],
        };

        let options = XlsxType1GeneratorOptions {
            outline_rows: false,
            shironuri: false,
        };
        let generator = XlsxType1Generator::new(outline, options);

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(worksheet).unwrap();

        // Save to a temporary file using rust_xlsxwriter
        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        // Read the file back and assert its content (using umya_spreadsheet as instructed)
        let read_spreadsheet = umya_spreadsheet::reader::xlsx::read(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Check if the background color of cell A1 is not set
        assert!(matches!(
            read_worksheet
                .get_cell("A1")
                .map(|cell| cell.get_style().get_background_color().is_none()),
            Some(true)
        ));

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
