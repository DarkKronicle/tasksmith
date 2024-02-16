use std::cmp::max;

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::{Line, Span, Text}};
use uuid::Uuid;

use crate::{data::TaskStatus, ui::tasklist::TableColumn};
use crate::data::Task;

use super::{render_row, RenderContext, RowEntry, FOLD_CLOSE, FOLD_OPEN};

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub task: Uuid,
    pub sub_tasks: Vec<RowEntry>,
}


impl TaskRow {

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
        let mut y_max = 0;
        let mut idx = context.index + 1;
        let folded = context.list.is_folded(idx - 1) && !self.sub_tasks.is_empty();
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
                        if !self.sub_tasks.is_empty() {
                            // Are there items to actually fold?
                            let fold_text: Span = if folded {
                                FOLD_CLOSE.into()
                            } else {
                                FOLD_OPEN.into()
                            };
                            lines.push(fold_text.style(context.theme.fold()));
                        }
                        lines.push(
                            Span::styled(&task.description, context.theme.text()),
                        );
                        let text: Text = Line::from(lines).into();
                        for line in &text.lines {
                            if context.y + y_offset >= area.height {
                                return (idx, max(y_max, y_offset));
                            }
                            buf.set_line(row_area.x + c_x + (context.depth * 2), row_area.y + y_offset, line, row_area.width);
                            y_offset += 1;
                        };
                        y_max = max(y_offset, y_max);
                    },
                    TableColumn::State => {
                        let (sequence, style) = match task.status {
                            TaskStatus::Blocked => {
                                ("", Style::default().fg(Color::Blue))
                            },
                            TaskStatus::Completed => {
                                ("", Style::default().fg(Color::Blue))
                            },
                            TaskStatus::Waiting => {
                                ("", Style::default().fg(Color::Blue))
                            },
                            TaskStatus::Deleted => {
                                ("", Style::default().fg(Color::Gray))
                            },
                            TaskStatus::Recurring => {
                                ("", Style::default().fg(Color::Blue))
                            },
                            TaskStatus::Pending => {
                                let urgency = task.urgency;
                                let block = if urgency > 9.0 {
                                    "◼◼◼"
                                } else if urgency > 6.0 {
                                    "◼◼"
                                } else if urgency > 3.0 {
                                    "◼"
                                } else {
                                    ""
                                };
                                (block, Style::default().fg(Color::Red))
                            }
                        };
                        let span: Span = Span::styled(sequence, style);
                        let text: Text = span.into();
                        let mut y_offset = 0;
                        let x_offset = (3 - sequence.chars().count()) as u16;
                        for line in &text.lines {
                            if context.y + y_offset >= area.height {
                                return (idx, max(y_max, y_offset));
                            }
                            buf.set_line(row_area.x + x_offset + c_x + (context.depth * 2), row_area.y + y_offset, line, row_area.width);
                            y_offset += 1;
                        };
                        y_max = max(y_offset, y_max);
                    }
                }
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
                    index: idx,
                    list: context.list,
                    task_map: context.task_map,
                });
                y_max += y_offset;
                idx = index;
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

}
