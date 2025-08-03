use crate::outline::Outline;
use anyhow::Result;
use clap::Args;
use rust_xlsxwriter::{Format, FormatBorder, Worksheet};

#[derive(Debug, Clone, Args)]
pub struct XlsxType2GeneratorOptions {
    /// group rows (default: no)
    #[arg(long, default_value_t = false)]
    pub outline_rows: bool,
    pub integrate_cells: Option<IntegrateCellsOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum IntegrateCellsOption {
    Colspan,
    Rowspan,
}

pub struct XlsxType2Generator {
    outline: Outline,
    options: XlsxType2GeneratorOptions,
}

impl XlsxType2Generator {
    pub fn new(outline: Outline, options: XlsxType2GeneratorOptions) -> Self {
        XlsxType2Generator { outline, options }
    }

    pub fn output_to_worksheet(&self, worksheet: &mut Worksheet) -> Result<()> {
        let max_level = self.outline.max_level();
        let max_value_length = self.outline.max_value_length();
        let mut row_index = 0;

        let header_format = Format::new().set_border(FormatBorder::Thin);
        let item_format = Format::new().set_border(FormatBorder::Thin);

        // Write key header and value headers
        let mut col_index = 0;
        for level in 1..=max_level {
            let header_text = self
                .outline
                .key_header
                .get((level - 1) as usize)
                .map_or("".to_string(), |s| s.clone());
            worksheet.write_string_with_format(
                row_index,
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
                row_index,
                col_index as u16,
                &header_text,
                &header_format,
            )?;
            col_index += 1;
        }
        row_index += 1;

        let item_first_row_index = row_index;

        for (item_index, item) in self.outline.item.iter().enumerate() {
            let _key_col_index = item.level - 1;

            // Apply borders based on Ruby logic
            for level in 1..=max_level {
                let mut format_for_level = Format::new();
                if (level as u32) <= item.level {
                    format_for_level = format_for_level.set_border_left(FormatBorder::Thin);
                }
                if ((level as u32) < item.level) || ((level as u32) == max_level) {
                    format_for_level = format_for_level.set_border_right(FormatBorder::Thin);
                }
                if ((level as u32) >= item.level) || (item_index == 0) {
                    format_for_level = format_for_level.set_border_top(FormatBorder::Thin);
                }
                if ((level as u32) > item.level) || (item_index == self.outline.item.len() - 1) {
                    format_for_level = format_for_level.set_border_bottom(FormatBorder::Thin);
                }
                worksheet.write_string_with_format(
                    row_index,
                    (level - 1) as u16,
                    if (level as u32) == item.level {
                        item.key.clone()
                    } else {
                        "".to_string()
                    },
                    &format_for_level,
                )?;
            }

            for i in 0..max_value_length {
                if let Some(value) = item.value.get(i) {
                    worksheet.write_string_with_format(
                        row_index,
                        (max_level + i as u32) as u16,
                        value,
                        &item_format,
                    )?;
                } else {
                    worksheet.write_string_with_format(
                        row_index,
                        (max_level + i as u32) as u16,
                        "",
                        &item_format,
                    )?;
                }
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

        // Integrate cells

        let mut format_for_integrate = Format::new();
        format_for_integrate = format_for_integrate.set_border_top(FormatBorder::Thin);
        format_for_integrate = format_for_integrate.set_border_left(FormatBorder::Thin);

        match self.options.integrate_cells {
            Some(IntegrateCellsOption::Colspan) => {
                let max_level = self.outline.max_level();

                for (item_index, item) in self.outline.item.iter().enumerate() {
                    if (item.level as u32) < max_level {
                        let text = &item.key;
                        worksheet.merge_range(
                            (item_index + 1) as u32,
                            (item.level - 1) as u16,
                            (item_index + 1) as u32,
                            (max_level - 1) as u16,
                            &text,
                            &format_for_integrate,
                        )?;
                    }
                }
            }
            Some(IntegrateCellsOption::Rowspan) => {
                for (item_index, item) in self.outline.item.iter().enumerate() {
                    let min_row_index = (item_index + 1) as u32;
                    let mut max_row_index = min_row_index;

                    for i in (item_index + 1)..self.outline.item.len() {
                        if (self.outline.item[i].level as u32) <= item.level {
                            break;
                        }
                        max_row_index = (i + 1) as u32;
                    }

                    if min_row_index != max_row_index {
                        let text = &item.key;
                        worksheet.merge_range(
                            min_row_index,
                            (item.level - 1) as u16,
                            max_row_index,
                            (item.level - 1) as u16,
                            &text,
                            &format_for_integrate,
                        )?;
                    }
                }
            }
            None => {}
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

    #[test]
    fn test_xlsx_type2_generator_basic() -> Result<()> {
        let mut outline = Outline::default();
        outline.key_header = vec!["Key Header 1".to_string(), "Key Header 2".to_string()];
        outline.value_header = vec!["Value Header 1".to_string(), "Value Header 2".to_string()];
        outline.add_item("Item 1", 1, vec!["Val1A".to_string(), "Val1B".to_string()]);
        outline.add_item("Item 1.1", 2, vec!["Val1.1A".to_string()]);
        outline.add_item("Item 2", 1, vec!["Val2A".to_string()]);

        let generator = XlsxType2Generator::new(
            outline,
            XlsxType2GeneratorOptions {
                outline_rows: false,
                integrate_cells: None,
            },
        );

        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(&mut worksheet).unwrap();

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Verify Header Row
        assert_eq!(read_worksheet.get_value((1, 1)).as_str(), "Key Header 1");
        assert_eq!(read_worksheet.get_value((2, 1)).as_str(), "Key Header 2");
        assert_eq!(read_worksheet.get_value((3, 1)).as_str(), "Value Header 1");
        assert_eq!(read_worksheet.get_value((4, 1)).as_str(), "Value Header 2");

        // Verify Data Row 1 (Item 1)
        assert_eq!(read_worksheet.get_value((1, 2)).as_str(), "Item 1");
        assert_eq!(read_worksheet.get_value((3, 2)).as_str(), "Val1A");
        assert_eq!(read_worksheet.get_value((4, 2)).as_str(), "Val1B");

        // Verify Data Row 2 (Item 1.1)
        assert_eq!(read_worksheet.get_value((2, 3)).as_str(), "Item 1.1");
        assert_eq!(read_worksheet.get_value((3, 3)).as_str(), "Val1.1A");

        // Verify Data Row 3 (Item 2)
        assert_eq!(read_worksheet.get_value((1, 4)).as_str(), "Item 2");
        assert_eq!(read_worksheet.get_value((3, 4)).as_str(), "Val2A");

        drop(temp_file);
        Ok(())
    }

    #[test]
    fn test_xlsx_type2_generator_outline_rows() -> Result<()> {
        let mut outline = Outline::default();
        outline.add_item("Item 1", 1, vec![]);
        outline.add_item("Subitem 1.1", 2, vec![]);
        outline.add_item("Subitem 1.2", 2, vec![]);
        outline.add_item("Item 2", 1, vec![]);
        outline.add_item("Subitem 2.1", 2, vec![]);

        let generator = XlsxType2Generator::new(
            outline,
            XlsxType2GeneratorOptions {
                outline_rows: true,
                integrate_cells: None,
            },
        );

        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(&mut worksheet).unwrap();

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Verify outline levels
        // assert_eq!(read_worksheet.get_row_dimension(&2).unwrap().get_outline_level(), &1);
        // assert_eq!(read_worksheet.get_row_dimension(&3).unwrap().get_outline_level(), &1);
        // assert_eq!(read_worksheet.get_row_dimension(&5).unwrap().get_outline_level(), &1);

        // Verify merge cell
        assert_eq!(read_worksheet.get_merge_cells().len(), 0);

        drop(temp_file);
        Ok(())
    }

    #[test]
    fn test_xlsx_type2_generator_integrate_cells_colspan() -> Result<()> {
        let mut outline = Outline::default();
        outline.key_header = vec![
            "Key Header 1".to_string(),
            "Key Header 2".to_string(),
            "Key Header 3".to_string(),
        ];
        outline.value_header = vec!["Value Header 1".to_string()];
        outline.add_item("Item 1", 1, vec!["Val1A".to_string()]);
        outline.add_item("Item 1.1", 2, vec!["Val1.1A".to_string()]);
        outline.add_item("Item 1.1.1", 3, vec!["Val1.1.1A".to_string()]);
        outline.add_item("Item 2", 1, vec!["Val2A".to_string()]);

        let generator = XlsxType2Generator::new(
            outline,
            XlsxType2GeneratorOptions {
                outline_rows: false,
                integrate_cells: Some(IntegrateCellsOption::Colspan),
            },
        );

        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(&mut worksheet).unwrap();

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Verify merge cell
        let merge_cells = read_worksheet.get_merge_cells();
        assert_eq!(
            merge_cells
                .into_iter()
                .map(|v| v.get_range())
                .collect::<Vec<_>>(),
            vec![
                "A2:C2".to_string(),
                "B3:C3".to_string(),
                "A5:C5".to_string()
            ]
        );
        assert_eq!(read_worksheet.get_value((1, 2)).as_str(), "Item 1");
        assert_eq!(read_worksheet.get_value((2, 3)).as_str(), "Item 1.1");
        assert_eq!(read_worksheet.get_value((1, 5)).as_str(), "Item 2");

        drop(temp_file);
        Ok(())
    }

    #[test]
    fn test_xlsx_type2_generator_integrate_cells_rowspan() -> Result<()> {
        let mut outline = Outline::default();
        outline.key_header = vec![
            "Key Header 1".to_string(),
            "Key Header 2".to_string(),
            "Key Header 3".to_string(),
        ];
        outline.value_header = vec!["Value Header 1".to_string()];
        outline.add_item("Item 1", 1, vec!["Val1A".to_string()]);
        outline.add_item("Item 1.1", 2, vec!["Val1.1A".to_string()]);
        outline.add_item("Item 1.1.1", 3, vec!["Val1.1.1A".to_string()]);
        outline.add_item("Item 1.2", 2, vec!["Val1.2A".to_string()]);
        outline.add_item("Item 2", 1, vec!["Val2A".to_string()]);

        let generator = XlsxType2Generator::new(
            outline,
            XlsxType2GeneratorOptions {
                outline_rows: false,
                integrate_cells: Some(IntegrateCellsOption::Rowspan),
            },
        );

        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        generator.output_to_worksheet(&mut worksheet).unwrap();

        let temp_file = NamedTempFile::with_suffix(".xlsx").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        workbook.save(&temp_path).unwrap();

        let read_spreadsheet = read_xlsx(&temp_path).unwrap();
        let read_worksheet = read_spreadsheet.get_sheet(&0).unwrap();

        // Verify merge cell
        let merge_cells = read_worksheet.get_merge_cells();
        assert_eq!(
            merge_cells
                .into_iter()
                .map(|v| v.get_range())
                .collect::<Vec<_>>(),
            vec!["A2:A5".to_string(), "B3:B4".to_string()]
        );
        assert_eq!(read_worksheet.get_value((1, 2)).as_str(), "Item 1");
        assert_eq!(read_worksheet.get_value((2, 3)).as_str(), "Item 1.1");

        drop(temp_file);
        Ok(())
    }
}
