use crate::outline::Outline;
use anyhow::Result;
use rust_xlsxwriter::{ColNum, Format, RowNum, Worksheet};

#[derive(Debug, Clone, Default)]
pub struct XlsxType0GeneratorOptions {
    /// If true, set the background color of all cells to white.
    pub shironuri: bool,
}

pub struct XlsxType0Generator {
    outline: Outline,
    options: XlsxType0GeneratorOptions,
}

impl XlsxType0Generator {
    pub fn new(outline: Outline, options: XlsxType0GeneratorOptions) -> Self {
        XlsxType0Generator { outline, options }
    }

    pub fn output_to_worksheet(&self, worksheet: &mut Worksheet) -> Result<()> {
        let mut row_index = 0;
        let max_value_length = self.outline.max_value_length();

        // Define a format for cells with thin borders
        let mut border_format = Format::new().set_border(rust_xlsxwriter::FormatBorder::Thin);
        // If shironuri is true, set the background color of border_format to white.
        if self.options.shironuri {
            border_format = border_format.set_background_color(rust_xlsxwriter::Color::White);
        }

        // If shironuri is true, set the background color of all cells to white.
        if self.options.shironuri {
            let cell_format = Format::new().set_background_color(rust_xlsxwriter::Color::White);
            worksheet.set_column_range_format(0, 16383, &cell_format)?;
        }

        // Header row
        let mut header_values = Vec::new();
        header_values.push(self.outline.key_header.first().cloned().unwrap_or_default());
        header_values.push("Outline Level".to_string());
        for s in self.outline.value_header.iter() {
            header_values.push(s.clone());
        }

        // Pad header_values with empty strings if necessary
        while header_values.len() < 2 + max_value_length {
            header_values.push("".to_string());
        }

        for (col_index, v) in header_values.iter().enumerate() {
            worksheet.write_with_format(
                row_index as RowNum,
                col_index as ColNum,
                v.clone(),
                &border_format,
            )?;
        }
        row_index += 1;

        // Data rows
        for item in &self.outline.item {
            let mut row_values: Vec<String> = Vec::new();
            row_values.push(item.key.clone());
            row_values.push(item.level.to_string());
            row_values.extend(item.value.iter().map(|s| s.to_string()));

            // Pad header_values with empty strings if necessary
            while row_values.len() < 2 + max_value_length {
                row_values.push("".to_string());
            }

            for (col_index, v) in row_values.iter().enumerate() {
                worksheet.write_with_format(
                    row_index as RowNum,
                    col_index as ColNum,
                    v.clone(),
                    &border_format,
                )?;
            }
            row_index += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outline::OutlineItem;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile; // Added for test

    #[test]
    fn test_xlsx_type0_generator() {
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

        let generator = XlsxType0Generator::new(outline, XlsxType0GeneratorOptions::default());

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

        // Verify Header Row
        assert_eq!(read_worksheet.get_value((1, 1)).as_str(), "Key");
        assert_eq!(read_worksheet.get_value((2, 1)).as_str(), "Outline Level");
        assert_eq!(read_worksheet.get_value((3, 1)).as_str(), "Value1");
        assert_eq!(read_worksheet.get_value((4, 1)).as_str(), "Value2");

        // Verify Header Row Borders
        let header_style_1_1 = read_worksheet.get_style((1, 1));
        assert_eq!(
            header_style_1_1
                .get_borders()
                .unwrap()
                .get_top()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );
        assert_eq!(
            header_style_1_1
                .get_borders()
                .unwrap()
                .get_bottom()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );
        assert_eq!(
            header_style_1_1
                .get_borders()
                .unwrap()
                .get_left()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );
        assert_eq!(
            header_style_1_1
                .get_borders()
                .unwrap()
                .get_right()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );

        // Verify Data Row 1
        assert_eq!(read_worksheet.get_value((1, 2)).as_str(), "Item 1");
        assert_eq!(read_worksheet.get_value((2, 2)).as_str(), "1");
        assert_eq!(read_worksheet.get_value((3, 2)).as_str(), "Val1A");
        assert_eq!(read_worksheet.get_value((4, 2)).as_str(), "Val1B");

        // Verify Data Row 2
        assert_eq!(read_worksheet.get_value((1, 3)).as_str(), "Item 2");
        assert_eq!(read_worksheet.get_value((2, 3)).as_str(), "2");
        assert_eq!(read_worksheet.get_value((3, 3)).as_str(), "Val2A");

        // Verify Data Row 3
        assert_eq!(read_worksheet.get_value((1, 4)).as_str(), "Item 3");
        assert_eq!(read_worksheet.get_value((2, 4)).as_str(), "1");
        assert_eq!(read_worksheet.get_value((3, 4)).as_str(), "Val3A");
        assert_eq!(read_worksheet.get_value((4, 4)).as_str(), "Val3B");
        assert_eq!(read_worksheet.get_value((5, 4)).as_str(), "Val3C");

        // Verify Data Row Borders (example for Item 1, cell (1,2))
        let data_style_1_2 = read_worksheet.get_style((1, 2));
        assert_eq!(
            data_style_1_2
                .get_borders()
                .unwrap()
                .get_top()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );
        assert_eq!(
            data_style_1_2
                .get_borders()
                .unwrap()
                .get_bottom()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );
        assert_eq!(
            data_style_1_2
                .get_borders()
                .unwrap()
                .get_left()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );
        assert_eq!(
            data_style_1_2
                .get_borders()
                .unwrap()
                .get_right()
                .get_border_style(),
            umya_spreadsheet::Border::BORDER_THIN
        );

        drop(temp_file);
    }

    #[test]
    fn test_xlsx_type0_generator_shironuri_enabled() {
        let outline = Outline {
            key_header: vec!["Key".to_string()],
            value_header: vec!["Value1".to_string(), "Value2".to_string()],
            item: vec![OutlineItem::new(
                "Item 1",
                1,
                vec!["Val1A".to_string(), "Val1B".to_string()],
            )],
        };

        let options = XlsxType0GeneratorOptions { shironuri: true };
        let generator = XlsxType0Generator::new(outline, options);

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
    fn test_xlsx_type0_generator_shironuri_disabled() {
        let outline = Outline {
            key_header: vec!["Key".to_string()],
            value_header: vec!["Value1".to_string(), "Value2".to_string()],
            item: vec![OutlineItem::new(
                "Item 1",
                1,
                vec!["Val1A".to_string(), "Val1B".to_string()],
            )],
        };

        let options = XlsxType0GeneratorOptions { shironuri: false };
        let generator = XlsxType0Generator::new(outline, options);

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
}
