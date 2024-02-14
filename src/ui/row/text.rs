use super::{RenderContext, RowEntry};
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span, Text}};

use super::{render_row, FOLD_CLOSE, FOLD_OPEN};



#[derive(Debug, Clone)]
pub struct TextRow
{
    pub sub_tasks: Vec<RowEntry>,
    pub text: String,
    pub sort_by: i8,
}



impl TextRow {
    pub fn render(
        &self, 
        area: Rect, 
        buf: &mut Buffer, 
        context: RenderContext,
    ) -> (usize, u16) {
        let row_area = Rect::new(
            area.x,
            area.y + context.y,
            area.width,
            1,
        );
        let mut idx = context.index + 1;
        let folded = context.list.is_folded(idx - 1) && !self.sub_tasks.is_empty();
        if context.list.cursor == idx - 1 {
            buf.set_style(row_area, context.theme.cursor());
        }
        let mut y_max = 0;
        let mut text_parts = vec![];
        let span: Span = self.text.clone().into();
        if !self.sub_tasks.is_empty() {
            // Are there items to actually fold?
            let fold_text: Span = if folded {
                FOLD_CLOSE.into()
            } else {
                FOLD_OPEN.into()
            };
            text_parts.push(fold_text.style(context.theme.fold()));
        }
        text_parts.push(span.clone());

        let text: Text = Line::from(text_parts).into();
        if idx - 1 >= context.list.focus {
            for line in &text {
                if context.y + y_max >= area.height {
                    return (idx, y_max)
                }
                buf.set_line(row_area.x + (context.depth * 2), row_area.y + y_max, line, row_area.width);
                y_max += 1;
            }
        }
        if !folded {
            for task in &self.sub_tasks {
                if context.y + y_max >= area.height {
                    return (idx, y_max)
                }
                let (index, y_offset) = render_row(task, area, buf, RenderContext {
                    y: context.y + y_max,
                    depth: context.depth + 1,
                    theme: context.theme.clone(),
                    widths: context.widths,
                    list: context.list,
                    index: idx,
                });
                idx = index;
                y_max += y_offset;
            }
        } else {
            idx = idx + self.len() - 1;
        }
        (idx, y_max)
    }

    pub fn len(&self) -> usize {
        let count: usize = self.sub_tasks.iter().map(|t| t.len()).sum();
        count + 1
    }

    pub fn new(text: String, sub_tasks: Vec<RowEntry>, sort_by: i8) -> TextRow {
        TextRow { 
            sub_tasks,
            text,
            sort_by,
        }
    }
}

