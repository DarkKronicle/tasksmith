use std::cmp::max;

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::{Line, Span, Text}};
use uuid::Uuid;

use crate::{data::TaskStatus, ui::tasklist::TableColumn};

use super::{FoldState, RenderContext, FOLD_CLOSE, FOLD_OPEN};

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub task: Uuid,
    pub fold_state: FoldState,
    pub depth: usize,
    pub idx: usize,
}


impl TaskRow {

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
        let mut y_max = 0;
        let mut idx = context.index + 1;
        if context.list.cursor == idx - 1 {
            buf.set_style(row_area, context.theme.cursor());
        }
        if idx > context.list.focus {
            let task = context.task_map.get(&self.task).unwrap();
            for (column, c_x, _width) in context.widths {
                match column {
                    TableColumn::Description => {
                        let mut y_offset = 0;
                        let mut lines = vec![];
                        match self.fold_state {
                            FoldState::NoChildren => {},
                            FoldState::Folded => {
                                let span: Span = FOLD_CLOSE.into();
                                lines.push(span);
                            }
                            FoldState::Open => {
                                let span: Span = FOLD_OPEN.into();
                                lines.push(span);
                            },
                        }
                        lines.push(
                            Span::styled(&task.description, context.theme.text()),
                        );
                        let text: Text = Line::from(lines).into();
                        for line in &text.lines {
                            if context.y + y_offset >= area.height {
                                return max(y_max, y_offset);
                            }
                            let depth: u16 = u16::try_from(self.depth * 2).unwrap();
                            buf.set_line(row_area.x + c_x + depth, row_area.y + y_offset, line, row_area.width);
                            y_offset += 1;
                        };
                        y_max = max(y_offset, y_max);
                    },
                    TableColumn::State => {
                        let (sequence, style) = task.status.get_display(task);
                        let span: Span = Span::styled(sequence.clone(), style);
                        let text: Text = span.into();
                        let mut y_offset = 0;
                        let x_offset = (3 - sequence.chars().count()) as u16;
                        for line in &text.lines {
                            if context.y + y_offset >= area.height {
                                return max(y_max, y_offset);
                            }
                            buf.set_line(row_area.x + x_offset + c_x + (context.depth * 2), row_area.y + y_offset, line, row_area.width);
                            y_offset += 1;
                        };
                        y_max = max(y_offset, y_max);
                    }
                }
            }
        }
        y_max
    }

}
