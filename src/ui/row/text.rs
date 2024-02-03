use super::{RenderContext, RowEntry};
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span, Text}};

use crate::ui::tasklist::TaskWidgetState;

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
        context: RenderContext,
    ) -> (usize, u16) {
        let mut idx = context.index + 1;
        if self.sub_tasks.is_empty() {
            return (idx, 0);
        }
        let row_area = Rect::new(
            area.x,
            area.y + context.y,
            area.width,
            1,
        );
        let mut y_max = 0;
        let mut text_parts = vec![];
        if !self.sub_tasks.is_empty() {
            // Are there items to actually fold?
            let fold_text: Span = if self.folded {
                FOLD_CLOSE.into()
            } else {
                FOLD_OPEN.into()
            };
            text_parts.push(fold_text.style(context.theme.fold()));
        }
        text_parts.push(self.text.clone());

        let text: Text = Line::from(text_parts).into();
        for line in &text {
            if context.y + y_max >= area.height {
                return (idx, y_max)
            }
            buf.set_line(row_area.x + (context.depth * 2), row_area.y + y_max, line, row_area.width);
            y_max += 1;
        }
        if !self.folded {
            for task in &self.sub_tasks {
                if context.y + y_max >= area.height {
                    return (idx, y_max)
                }
                let (index, y_offset) = render_row(task, area, buf, state, RenderContext {
                    y: context.y + y_max,
                    depth: context.depth + 1,
                    theme: context.theme.clone(),
                    widths: context.widths,
                    index: idx,
                });
                idx = index;
                y_max += y_offset;
            }
        } else {
            idx = context.index + self.len() - 1;
        }
        (idx, y_max)
    }

    pub fn len(&self) -> usize {
        let count: usize = self.sub_tasks.iter().map(|t| t.len()).sum();
        count + 1
    }
}

