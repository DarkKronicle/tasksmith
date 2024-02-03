use super::RowEntry;
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span, Text}};

use crate::ui::{style::SharedTheme, tasklist::{TableColumn, TaskWidgetState}};

use super::{render_row, FOLD_CLOSE, FOLD_OPEN};



#[derive(Debug)]
pub struct TextRow<'a> {
    pub sub_tasks: Vec<RowEntry<'a>>,
    pub text: Span<'a>,
    pub sort_by: i8,
    pub folded: bool,
}



impl<'a> TextRow<'a> {
    pub fn render(
        &self, 
        area: Rect, 
        buf: &mut Buffer, 
        state: &mut TaskWidgetState, 
        y: u16, 
        depth: u16, 
        theme: SharedTheme, 
        widths: &Vec<(TableColumn, u16, u16)>,
        index: usize,
    ) -> (usize, u16) {
        let mut idx = index + 1;
        if self.sub_tasks.len() == 0 {
            return (idx, 0);
        }
        let row_area = Rect::new(
            area.x,
            area.y + (y as u16),
            area.width,
            1,
        );
        let mut y_max = 0;
        let mut text_parts = vec![];
        if self.sub_tasks.len() > 0 {
            // Are there items to actually fold?
            let fold_text: Span = if self.folded {
                FOLD_CLOSE.into()
            } else {
                FOLD_OPEN.into()
            };
            text_parts.push(fold_text.style(theme.fold()));
        }
        text_parts.push(self.text.clone().into());

        let text: Text = Line::from(text_parts).into();
        for line in &text {
            if y + y_max >= area.height {
                return (idx, y_max)
            }
            buf.set_line(row_area.x + (depth * 2), row_area.y + y_max as u16, line, row_area.width);
            y_max += 1;
        }
        if !self.folded {
            for task in &self.sub_tasks {
                if y + y_max >= area.height {
                    return (idx, y_max)
                }
                let (index, y_offset) = render_row(task, area, buf, state, y + y_max, depth, theme.clone(), widths, idx);
                idx = index;
                y_max += y_offset;
            }
        } else {
        
        }
        (idx, y_max)
    }

    pub fn len(self: &Self) -> usize {
        let count: usize = self.sub_tasks.iter().map(|t| t.len()).sum();
        return count + 1;
    }
}

