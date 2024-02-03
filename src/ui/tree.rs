use std::{cmp::max, collections::{HashMap}};

use crate::{app::App, data::{Task, TaskStatus}};
use petgraph::{graph::{NodeIndex}, Direction, Graph};
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::{Line, Span, Text}};
use uuid::Uuid;
use strum::IntoEnumIterator;

use super::{style::SharedTheme, task::{TableColumn, TaskWidgetState}};


const FOLD_OPEN: &str = " ";
const FOLD_CLOSE: &str = " ";


#[derive(Debug)]
pub struct TaskRow<'a> {
    task: &'a Task,
    pub sub_tasks: Vec<RowEntry<'a>>,
    pub folded: bool,
}

#[derive(Debug)]
pub struct RootRow<'a> {
    pub sub_tasks: Vec<RowEntry<'a>>,
}

#[derive(Debug)]
pub struct TextRow<'a> {
    pub sub_tasks: Vec<RowEntry<'a>>,
    pub text: Span<'a>,
    pub sort_by: i8,
    pub folded: bool,
}

pub fn render_row(
    row: &RowEntry, 
    area: Rect, 
    buf: &mut Buffer, 
    state: &mut TaskWidgetState, 
    y: u16, 
    depth: u16, 
    theme: SharedTheme, 
    widths: &Vec<(TableColumn, u16, u16)>
) -> u16 {
    match row {
        RowEntry::Task(t) => {
            t.render(area, buf, state, y, depth, theme.clone(), widths)
        },
        RowEntry::Text(t) => {
            t.render(area, buf, state, y, depth, theme.clone(), widths)
        },
        _ => {
            0
        }
    }
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
        widths: &Vec<(TableColumn, u16, u16)>
    ) -> u16 {
        if self.sub_tasks.len() == 0 {
            return 0;
        }
        let row_area = Rect::new(
            area.x,
            area.y + y,
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
                return y_max
            }
            buf.set_line(row_area.x + (depth * 2), row_area.y + y_max as u16, line, row_area.width);
            y_max += 1;
        }
        if !self.folded {
            for task in &self.sub_tasks {
                if y + y_max >= area.height {
                    return y_max
                }
                y_max += render_row(task, area, buf, state, y + y_max, depth, theme.clone(), widths);
            }
        }
        y_max
    }
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


#[derive(Debug)]
pub enum RowEntry<'a> {
    Root(RootRow<'a>),
    Text(TextRow<'a>),
    Task(TaskRow<'a>),
}

pub struct TaskGraph {
    graph: Graph<Option<Uuid>, ()>,
    root: NodeIndex,
}

// TODO: When I'm good at rust look at this again, I think there's a better way to do this
impl TaskGraph {

    pub fn empty() -> Self {
        let mut graph = Graph::new();
        let root = graph.add_node(None);
        Self {
            graph,
            root,
        }
    }

    pub fn new(app: &App) -> Self {
        let mut graph = Graph::<Option<Uuid>, ()>::new();
        let mut node_map: HashMap<Uuid, NodeIndex> = HashMap::new();
        let root = graph.add_node(None);

        for (uuid, task) in app.tasks.iter() {
            let task_index = *node_map.entry(*uuid).or_insert_with(|| {
                graph.add_node(Some(*uuid))
            });
            
            if let Some(parent_uuid) = &task.sub_of {
                let parent_index = node_map.entry(*parent_uuid).or_insert_with(|| {
                    
                    graph.add_node(Some(*parent_uuid))
                });
                graph.add_edge(*parent_index, task_index, ());
            } else {
                graph.add_edge(root, task_index, ());
            }
        }

        TaskGraph {
            graph,
            root,
        }
    }

    pub fn get_tasks<'b>(&'b self, app: &'b App, node: NodeIndex, separate: bool) -> Vec<RowEntry> {
        let neighbors: Vec<_> = self.graph.neighbors_directed(node, Direction::Outgoing).collect();

        // let mut tasks = vec![];
        let mut status_map: HashMap<TaskStatus, Vec<RowEntry>> = TaskStatus::iter().map(|t| (t, vec![])).collect();

        for n in neighbors {
            let task_uuid = self.graph.node_weight(n).unwrap();
            let task = &app.tasks[&task_uuid.unwrap()];
            status_map.get_mut(&task.status).unwrap().push(
                RowEntry::Task(TaskRow {
                    task,
                    sub_tasks: {
                        let mut tasks = self.get_tasks(app, n, false);
                        Self::sort_rows(&mut tasks);
                        tasks
                    },
                    folded: false
                }));
        };
        let mut sorted_statuses = TaskStatus::iter().collect::<Vec<_>>();
        sorted_statuses.sort();
        let mut rows: Vec<RowEntry> = if separate {
            status_map.into_iter().map(|(status, mut entries)| {
                let span: Span = status.to_string().into();
                let text = span.style(Style::default().fg(Color::Green));
                Self::sort_rows(&mut entries);
                RowEntry::Text(TextRow {
                    sub_tasks: entries,
                    text,
                    sort_by: sorted_statuses.iter().position(|s| s == &status).unwrap() as i8,
                    folded: status == TaskStatus::Deleted
                })
            }).collect()
        } else {
            status_map.into_iter().map(|(_, entries)| entries).flatten().collect()
        };
        Self::sort_rows(&mut rows);
        rows
    }

    pub fn sort_rows(rows: &mut Vec<RowEntry>) {
        rows.sort_by(|a, b| {
            if let RowEntry::Task(a) = a {
                if let RowEntry::Task(b) = b {
                    if a.task.status == b.task.status {
                        return b.task.urgency.partial_cmp(&a.task.urgency).expect("Invalid urgency");
                    }
                    return a.task.status.partial_cmp(&b.task.status).expect("Invalid status");
                }
            } else if let RowEntry::Text(a) = a {
                if let RowEntry::Text(b) = b {
                    return a.sort_by.cmp(&b.sort_by);
                }
            }
            return std::cmp::Ordering::Equal;
        });
    }

    pub fn get_root<'b>(&'b self, app: &'b App) -> RootRow {
        RootRow {
            sub_tasks: self.get_tasks(app, self.root, true)
        }
    }

}
