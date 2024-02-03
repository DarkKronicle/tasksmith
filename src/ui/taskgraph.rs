use std::collections::HashMap;

use crate::{app::App, data::{Task, TaskStatus}};
use petgraph::{graph::NodeIndex, Direction, Graph};
use ratatui::{style::{Color, Style}, text::Span};
use uuid::Uuid;
use strum::IntoEnumIterator;

use super::row::{task::TaskRow, text::TextRow, RootRow, RowEntry};

#[derive(Debug, Clone)]
pub struct TaskGraph {
    graph: Graph<Option<Uuid>, ()>,
    root: NodeIndex,
}

impl TaskGraph {

    pub fn empty() -> Self {
        let mut graph = Graph::new();
        let root = graph.add_node(None);
        Self {
            graph,
            root,
        }
    }

    pub fn new(app: &HashMap<Uuid, Task>) -> Self {
        let mut graph = Graph::<Option<Uuid>, ()>::new();
        let mut node_map: HashMap<Uuid, NodeIndex> = HashMap::new();
        let root = graph.add_node(None);

        for (uuid, task) in app.iter() {
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

    pub fn get_tasks<'b>(&'b self, tasks: &'b HashMap<Uuid, Task>, node: NodeIndex, separate: bool) -> Vec<RowEntry> {
        let neighbors: Vec<_> = self.graph.neighbors_directed(node, Direction::Outgoing).collect();

        // let mut tasks = vec![];
        let mut status_map: HashMap<TaskStatus, Vec<RowEntry>> = TaskStatus::iter().map(|t| (t, vec![])).collect();

        for n in neighbors {
            let task_uuid = self.graph.node_weight(n).unwrap();
            let task = &tasks[&task_uuid.unwrap()];
            status_map.get_mut(&task.status).unwrap().push(
                RowEntry::Task(TaskRow {
                    task,
                    sub_tasks: {
                        let mut tasks = self.get_tasks(tasks, n, false);
                        Self::sort_rows(&mut tasks);
                        tasks
                    },
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
                })
            }).collect()
        } else {
            status_map.into_values().flatten().collect()
        };
        Self::sort_rows(&mut rows);
        rows
    }

    pub fn sort_rows(rows: &mut [RowEntry]) {
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
            std::cmp::Ordering::Equal
        });
    }

    pub fn get_root<'b>(&'b self, tasks: &'b HashMap<Uuid, Task>) -> RootRow {
        RootRow {
            sub_tasks: self.get_tasks(tasks, self.root, true)
        }
    }

}
