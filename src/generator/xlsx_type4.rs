use std::rc::Rc;

use crate::generator::base::IntegrateCellsOption;
use crate::outline::{Outline, OutlineTree};
use anyhow::Result;
use clap::Args;
use rust_xlsxwriter::{Format, FormatBorder, Worksheet};

#[derive(Debug, Clone, Args)]
pub struct XlsxType4GeneratorOptions {
    /// integrate key cells (specify 'colspan', 'rowspan' or 'both')
    #[arg(long)]
    pub integrate_cells: Option<IntegrateCellsOption>,
}

pub struct XlsxType4Generator {
    outline: Outline,
    options: XlsxType4GeneratorOptions,
}

impl XlsxType4Generator {
    pub fn new(outline: Outline, options: XlsxType4GeneratorOptions) -> Self {
        XlsxType4Generator { outline, options }
    }

    pub fn output_to_worksheet(&self, worksheet: &mut Worksheet) -> Result<()> {
        let max_level = self.outline.max_level() as usize;
        let max_value_length = self.outline.max_value_length();

        let mut row_index: u32 = 0;

        let header_format = Format::new().set_border(FormatBorder::Thin);
        let item_format = Format::new().set_border(FormatBorder::Thin);

        // Write key header and value headers
        let mut col_index = 0;
        for level in 1..=max_level {
            let header_text = self
                .outline
                .key_header
                .get(level - 1)
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

        // Rowspan tracking per level: remember active parent node (group) and its start row
        let mut active_parent: Vec<Option<usize>> = vec![None; max_level];
        let mut active_start_row: Vec<Option<u32>> = vec![None; max_level];
        let mut active_value: Vec<Option<String>> = vec![None; max_level];

        let tree = self.outline.to_tree();
        for node_rc in OutlineTree::descendants(&tree) {
            if !node_rc.borrow().is_leaf() {
                continue;
            }

            // Build key cells across levels using ancestors and prev boundary
            let mut key_cells: Vec<Option<String>> = vec![None; max_level];
            for c_node in std::iter::once(node_rc.clone()).chain(OutlineTree::ancestors(&node_rc)) {
                if let Some(c_item) = c_node.borrow().item() {
                    key_cells[c_item.level as usize - 1] = Some(c_item.key.clone())
                }
                if OutlineTree::prev(&c_node).is_some() {
                    break;
                }
            }

            let mut value_cell = node_rc
                .borrow()
                .item()
                .map(|v| v.value.iter().map(|u| Some(u.clone())).collect())
                .unwrap_or(vec![]);
            value_cell.resize(max_value_length, None);

            for (col_index, v) in key_cells.iter().chain(value_cell.iter()).enumerate() {
                worksheet.write_string_with_format(
                    row_index,
                    col_index as u16,
                    v.clone().unwrap_or_default(),
                    &item_format,
                )?;
            }

            {
                let mut descendants = vec![node_rc.clone()];
                for c_node in OutlineTree::ancestors(&node_rc) {
                    if let Some(item) = c_node.borrow().item() {
                        if item.level > 0 {
                            let mut format = Format::new();
                            if descendants.iter().any(|v| OutlineTree::prev(v).is_some()) {
                                format = format.set_border_top(FormatBorder::Thin);
                            }
                            if descendants.iter().any(|v| OutlineTree::next(v).is_some()) {
                                format = format.set_border_bottom(FormatBorder::Thin);
                            }
                            worksheet.set_cell_format(row_index, item.level as u16 - 1, &format)?;
                        }
                    }
                    descendants.push(c_node);
                }
            }

            if self.options.integrate_cells == Some(IntegrateCellsOption::Colspan)
                || self.options.integrate_cells == Some(IntegrateCellsOption::Both)
            {
                if let Some(item) = node_rc.borrow().item() {
                    if item.level < max_level as u32 {
                        let val = key_cells[item.level as usize - 1].as_deref().unwrap_or("");
                        worksheet.merge_range(
                            row_index,
                            item.level as u16 - 1,
                            row_index,
                            max_level as u16 - 1,
                            val,
                            &item_format,
                        )?;
                    }
                }
            }

            if self.options.integrate_cells == Some(IntegrateCellsOption::Rowspan)
                || self.options.integrate_cells == Some(IntegrateCellsOption::Both)
            {
                // For each ancestor level, use the ancestor node pointer identity as group key.
                // We rely on Rc pointer address via as_ptr() cast to usize, safe for grouping identity here.
                let mut ancestors: Vec<_> = OutlineTree::ancestors(&node_rc).into_iter().collect();
                // ensure index 0..=level-1 alignment by pushing current node at front
                ancestors.insert(0, node_rc.clone());

                for anc in ancestors.into_iter() {
                    if let Some(item) = anc.borrow().item() {
                        let level_idx = item.level as usize - 1;
                        let key = Rc::as_ptr(&anc) as usize;
                        match (active_parent[level_idx], active_start_row[level_idx]) {
                            (Some(prev_key), Some(start_row)) if prev_key != key => {
                                let end_row = row_index - 1;
                                if end_row > start_row {
                                    let val = active_value[level_idx].as_deref().unwrap_or("");
                                    worksheet.merge_range(
                                        start_row,
                                        level_idx as u16,
                                        end_row,
                                        level_idx as u16,
                                        val,
                                        &item_format,
                                    )?;
                                }
                                active_parent[level_idx] = Some(key);
                                active_start_row[level_idx] = Some(row_index);
                                active_value[level_idx] = Some(item.key.clone());
                            }
                            (None, _) => {
                                active_parent[level_idx] = Some(key);
                                active_start_row[level_idx] = Some(row_index);
                                active_value[level_idx] = Some(item.key.clone());
                            }
                            _ => { /* same group continues, do nothing */ }
                        }
                    }
                }
            }

            for col_index in max_level..(max_level + max_value_length - 1) {
                worksheet.set_cell_format(row_index, col_index as u16, &item_format)?;
            }

            row_index += 1;
        }

        // Flush pending rowspans at EOF
        if self.options.integrate_cells == Some(IntegrateCellsOption::Rowspan)
            || self.options.integrate_cells == Some(IntegrateCellsOption::Both)
        {
            for (level_idx, start_row_opt) in active_start_row.iter().enumerate() {
                if let Some(start_row) = start_row_opt {
                    let start_row = *start_row;
                    let end_row = row_index - 1;
                    if end_row > start_row {
                        let val = active_value[level_idx].as_deref().unwrap_or("");
                        worksheet.merge_range(
                            start_row,
                            level_idx as u16,
                            end_row,
                            level_idx as u16,
                            val,
                            &item_format,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::outline::OutlineItem;

    use super::*;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;
    use umya_spreadsheet::reader::xlsx::read as read_xlsx;

    #[test]
    fn test_output_worksheet_basic_and_no_merge() -> Result<()> {
        // reference_outline equivalent
        let outline = Outline {
            key_header: vec!["H1".into(), "H2".into(), "H3".into()],
            value_header: vec!["H(1)".into(), "H(2)".into()],
            item: vec![
                OutlineItem::new("1", 1, vec![]),
                OutlineItem::new("1.1", 2, vec!["1.1(1)".into(), "1.1(2)".into()]),
                OutlineItem::new("1.2", 2, vec![]),
                OutlineItem::new("1.2.1", 3, vec!["1.2.1(1)".into(), "1.2.1(2)".into()]),
            ],
        };

        let gen = XlsxType4Generator::new(
            outline.clone(),
            XlsxType4GeneratorOptions {
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

        // Header row values
        assert_eq!(ws.get_value((1, 1)).as_str(), "H1");
        assert_eq!(ws.get_value((2, 1)).as_str(), "H2");
        assert_eq!(ws.get_value((3, 1)).as_str(), "H3");
        assert_eq!(ws.get_value((4, 1)).as_str(), "H(1)");
        assert_eq!(ws.get_value((5, 1)).as_str(), "H(2)");

        // First data row (Ruby expected): ["1", "1.1", nil, "1.1(1)", "1.1(2)"]
        // In umya_spreadsheet 1-based (col,row): A2..E2
        assert_eq!(ws.get_value((1, 2)).as_str(), "1"); // A2
        assert_eq!(ws.get_value((2, 2)).as_str(), "1.1"); // B2
        assert_eq!(ws.get_value((3, 2)).as_str(), ""); // C2
        assert_eq!(ws.get_value((4, 2)).as_str(), "1.1(1)"); // D2
        assert_eq!(ws.get_value((5, 2)).as_str(), "1.1(2)"); // E2

        // Second data row (Ruby expected): [nil, "1.2", "1.2.1", "1.2.1(1)", "1.2.1(2)"]
        assert_eq!(ws.get_value((1, 3)).as_str(), ""); // A3
        assert_eq!(ws.get_value((2, 3)).as_str(), "1.2"); // B3
        assert_eq!(ws.get_value((3, 3)).as_str(), "1.2.1"); // C3
        assert_eq!(ws.get_value((4, 3)).as_str(), "1.2.1(1)"); // D3
        assert_eq!(ws.get_value((5, 3)).as_str(), "1.2.1(2)"); // E3

        assert_eq!(ws.get_merge_cells().len(), 0);

        // second case from Ruby: only keys, no merges
        let outline2 = Outline {
            key_header: vec!["H1".into(), "H2".into(), "H3".into()],
            value_header: vec![],
            item: vec![
                OutlineItem::new("1", 1, vec![]),
                OutlineItem::new("1.1", 2, vec![]),
                OutlineItem::new("1.2", 2, vec![]),
                OutlineItem::new("1.2.1", 3, vec![]),
            ],
        };

        let gen2 = XlsxType4Generator::new(
            outline2.clone(),
            XlsxType4GeneratorOptions {
                integrate_cells: None,
            },
        );

        let mut wb2 = Workbook::new();
        let ws2 = wb2.add_worksheet();
        gen2.output_to_worksheet(ws2)?;

        let tmp2 = NamedTempFile::with_suffix(".xlsx").unwrap();
        let path2 = tmp2.path().to_path_buf();
        wb2.save(&path2).unwrap();

        let book2 = read_xlsx(&path2).unwrap();
        let ws_2 = book2.get_sheet(&0).unwrap();

        assert_eq!(ws_2.get_value((1, 1)).as_str(), "H1");
        assert_eq!(ws_2.get_value((2, 1)).as_str(), "H2");
        assert_eq!(ws_2.get_value((3, 1)).as_str(), "H3");

        assert_eq!(ws_2.get_value((1, 2)).as_str(), "1");
        assert_eq!(ws_2.get_value((2, 2)).as_str(), "1.1");
        assert_eq!(ws_2.get_value((3, 2)).as_str(), "");

        assert_eq!(ws_2.get_value((1, 3)).as_str(), "");
        assert_eq!(ws_2.get_value((2, 3)).as_str(), "1.2");
        assert_eq!(ws_2.get_value((3, 3)).as_str(), "1.2.1");

        assert_eq!(ws_2.get_merge_cells().len(), 0);

        Ok(())
    }

    #[test]
    fn test_output_worksheet_with_integrate_cells() -> Result<()> {
        // base outline same as first case
        let outline = {
            let mut o = Outline::new();
            o.key_header = vec!["H1".into(), "H2".into(), "H3".into()];
            o.value_header = vec!["H(1)".into(), "H(2)".into()];
            o.add_item("1", 1, vec![]);
            o.add_item("1.1", 2, vec!["1.1(1)".into(), "1.1(2)".into()]);
            o.add_item("1.2", 2, vec![]);
            o.add_item("1.2.1", 3, vec!["1.2.1(1)".into(), "1.2.1(2)".into()]);
            o
        };

        // Colspan
        let gen_col = XlsxType4Generator::new(
            outline.clone(),
            XlsxType4GeneratorOptions {
                integrate_cells: Some(IntegrateCellsOption::Colspan),
            },
        );
        let mut wb1 = Workbook::new();
        let ws1 = wb1.add_worksheet();
        gen_col.output_to_worksheet(ws1)?;
        let tmp1 = NamedTempFile::with_suffix(".xlsx").unwrap();
        let path1 = tmp1.path().to_path_buf();
        wb1.save(&path1).unwrap();
        let b1 = read_xlsx(&path1).unwrap();
        let ws_1 = b1.get_sheet(&0).unwrap();
        let merged1: Vec<String> = ws_1
            .get_merge_cells()
            .iter()
            .map(|m| m.get_range())
            .collect();
        assert_eq!(merged1, vec!["B2:C2".to_string()]);

        // Rowspan
        let gen_row = XlsxType4Generator::new(
            outline.clone(),
            XlsxType4GeneratorOptions {
                integrate_cells: Some(IntegrateCellsOption::Rowspan),
            },
        );
        let mut wb2 = Workbook::new();
        let ws2 = wb2.add_worksheet();
        gen_row.output_to_worksheet(ws2)?;
        let tmp2 = NamedTempFile::with_suffix(".xlsx").unwrap();
        let path2 = tmp2.path().to_path_buf();
        wb2.save(&path2).unwrap();
        let b2 = read_xlsx(&path2).unwrap();
        let ws_2 = b2.get_sheet(&0).unwrap();
        let merged2: Vec<String> = ws_2
            .get_merge_cells()
            .iter()
            .map(|m| m.get_range())
            .collect();
        assert_eq!(merged2, vec!["A2:A3".to_string()]);

        // Both on second outline case (only keys)
        let outline2 = {
            let mut o = Outline::new();
            o.key_header = vec!["H1".into(), "H2".into(), "H3".into()];
            o.value_header = vec![];
            o.add_item("1", 1, vec![]);
            o.add_item("1.1", 2, vec![]);
            o.add_item("1.1.1", 3, vec![]);
            o.add_item("1.1.2", 3, vec![]);
            o
        };
        let gen_both = XlsxType4Generator::new(
            outline2,
            XlsxType4GeneratorOptions {
                integrate_cells: Some(IntegrateCellsOption::Both),
            },
        );
        let mut wb3 = Workbook::new();
        let ws3 = wb3.add_worksheet();
        gen_both.output_to_worksheet(ws3)?;
        let tmp3 = NamedTempFile::with_suffix(".xlsx").unwrap();
        let path3 = tmp3.path().to_path_buf();
        wb3.save(&path3).unwrap();
        let b3 = read_xlsx(&path3).unwrap();
        let ws_3 = b3.get_sheet(&0).unwrap();
        let mut merged3: Vec<String> = ws_3
            .get_merge_cells()
            .iter()
            .map(|m| m.get_range())
            .collect();
        merged3.sort();
        assert_eq!(merged3, vec!["A2:A3".to_string(), "B2:B3".to_string()]);

        Ok(())
    }
}
