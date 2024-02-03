use std::cmp::max;

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::{Line, Span, Text}};

use crate::{data::TaskStatus, ui::{style::SharedTheme, tasklist::{TableColumn, TaskWidgetState}}};
use crate::data::Task;

use super::{render_row, RowEntry, FOLD_CLOSE, FOLD_OPEN};

#[derive(Debug)]
pub struct TaskRow<'a> {
    pub task: &'a Task,
    pub sub_tasks: Vec<RowEntry<'a>>,
    pub folded: bool,
}


impl<'a> TaskRow<'a> {

    pub fn render(
        &self, 
        area: Rect, 
        buf: &mut Buffer, 
        state: &mut TaskWidgetState, 
        y: u16, 
        depth: u16, 
        theme: SharedTheme, 
        widths: &Vec<(TableColumn, u16, u16)>
    ) -> u16 {
        let row_area = Rect::new(
            area.x,
            area.y + y,
            area.width,
            1,
        );
        let mut y_max = 0;
        for (column, c_x, _width) in widths {
            match column {
                TableColumn::Description => {
                    let mut y_offset = 0;
                    let mut lines = vec![];
                    if self.sub_tasks.len() > 0 {
                        // Are there items to actually fold?
                        let fold_text: Span = if self.folded {
                            FOLD_CLOSE.into()
                        } else {
                            FOLD_OPEN.into()
                        };
                        lines.push(fold_text.style(theme.fold()));
                    }
                    lines.push(
                        Span::styled(&self.task.description, theme.text()),
                    );
                    let text: Text = Line::from(lines).into();
                    for line in &text.lines {
                        if y + y_offset >= area.height {
                            return max(y_max, y_offset);
                        }
                        buf.set_line(row_area.x + c_x + (depth * 2), row_area.y + y_offset as u16, line, row_area.width);
                        y_offset += 1;
                    };
                    y_max = max(y_offset, y_max);
                },
                TableColumn::State => {
                    let (sequence, style) = match self.task.status {
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
                            let urgency = self.task.urgency;
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
                        if y + y_offset >= area.height {
                            return max(y_max, y_offset);
                        }
                        buf.set_line(row_area.x + x_offset + c_x + (depth * 2), row_area.y + y_offset as u16, line, row_area.width);
                        y_offset += 1;
                    };
                    y_max = max(y_offset, y_max);
                }
            }
        }
        if !self.folded {
            for task in &self.sub_tasks {
                if y + y_max >= area.height {
                    return y_max
                }
                y_max += render_row(task, area, buf, state, y + y_max, depth + 1, theme.clone(), widths);
            }
        }
        y_max
    }
}
