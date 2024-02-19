use super::{FoldState, RenderContext, RowEntry};
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span, Text}};

use super::{render_row, FOLD_CLOSE, FOLD_OPEN};



#[derive(Debug, Clone)]
pub struct TextRow
{
    pub text: String,
    pub sort_by: i8,
    pub fold_state: FoldState,
    pub depth: usize,
}



impl TextRow {
    pub fn render(
        &self, 
        area: Rect, 
        buf: &mut Buffer, 
        context: RenderContext,
    ) -> u16 {
        let row_area = Rect::new(
            area.x,
            area.y + context.y,
            area.width,
            1,
        );
        let mut idx = context.index + 1;
        let folded = context.list.is_folded(idx - 1);
        if context.list.cursor == idx - 1 {
            buf.set_style(row_area, context.theme.cursor());
        }
        let mut y_max = 0;
        let mut text_parts = vec![];
        let span: Span = self.text.clone().into();
        match self.fold_state {
            FoldState::NoChildren => {},
            FoldState::Folded => {
                let span: Span = FOLD_CLOSE.into();
                text_parts.push(span);
            }
            FoldState::Open => {
                let span: Span = FOLD_OPEN.into();
                text_parts.push(span);
            },
        }
        text_parts.push(span.clone());

        let text: Text = Line::from(text_parts).into();
        if idx > context.list.focus {
            for line in &text {
                if context.y + y_max >= area.height {
                    return y_max
                }
                buf.set_line(row_area.x + (context.depth * 2), row_area.y + y_max, line, row_area.width);
                y_max += 1;
            }
        }
        y_max
    }


    pub fn new(text: String, sort_by: i8, depth: usize, fold_state: FoldState) -> TextRow {
        TextRow { 
            text,
            sort_by,
            depth,
            fold_state,
        }
    }
}

