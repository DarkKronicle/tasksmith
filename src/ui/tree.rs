use std::{collections::{HashMap}};

use crate::{app::App, data::Task};
use color_eyre::Result;
use petgraph::{data::Build, graph::{NodeIndex}, Direction, Graph};
use ratatui::{buffer::Buffer, layout::Rect, text::Text};
use uuid::Uuid;
use petgraph::visit::{Walker};

use super::task::TaskWidgetState;


pub enum SortType {
    Urgency,
}

pub enum RowAction {

}

pub enum RenderContext {
    List,
}


trait Row {

    fn compare(&self, other: &dyn Row, sorttype: SortType) -> std::cmp::Ordering;

    fn action(&self, action: RowAction) -> Result<()>;
    
}

#[derive(Debug)]
pub struct TaskRow<'a> {
    task: &'a Task,
    pub sub_tasks: Vec<RowEntry<'a>>,
}

#[derive(Debug)]
pub struct RootRow<'a> {
    pub sub_tasks: Vec<RowEntry<'a>>,
}

impl<'a> Row for RootRow<'a> {
    fn compare(&self, _other: &dyn Row, _sorttype: SortType) -> std::cmp::Ordering {
        todo!()
    }

    fn action(&self, _action: RowAction) -> Result<()> {
        todo!()
    }
}

impl<'a> Row for TaskRow<'a> {
    fn compare(&self, _other: &dyn Row, _sorttype: SortType) -> std::cmp::Ordering {
        todo!()
    }

    fn action(&self, _action: RowAction) -> Result<()> {
        todo!()
    }
}

impl<'a> TaskRow<'a> {
    fn from(task: &'a Task) -> Self {
        TaskRow {
            task,
            sub_tasks: vec![]
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &mut TaskWidgetState, x: u16, y: u16, depth: u16) -> u16 {
        let row_area = Rect::new(
            area.x + x,
            area.y + y, 
            area.width,
            1,
        );
        let text: Text = self.task.description.as_str().into();
        let mut y_offset = 0;
        for (j, line) in text.lines.iter().enumerate() {
            buf.set_line(row_area.x, row_area.y + j as u16, line, row_area.width);
            y_offset += 1;
        };
        for task in &self.sub_tasks {
            match task {
                RowEntry::Task(t) => {
                    y_offset += t.render(row_area, buf, state, 2, y_offset, depth + 1);
                },
                _ => {},
            }
        }
        y_offset
    }
}


#[derive(Debug)]
pub enum RowEntry<'a> {
    Root(RootRow<'a>),
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
            // NOTE: wth, this ^ clone is really important. Should revisit this
            
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

    pub fn get_tasks<'b>(&'b self, app: &'b App, node: NodeIndex) -> Vec<RowEntry> {
        let mut tasks = vec![];
        let neighbors: Vec<_> = self.graph.neighbors_directed(node, Direction::Outgoing).collect();

        for n in neighbors {
            let task_uuid = self.graph.node_weight(n).unwrap();
            let task = &app.tasks[&task_uuid.unwrap()];
            tasks.push(RowEntry::Task(TaskRow {
                task,
                sub_tasks: self.get_tasks(app, n)
            }));
        };
        tasks
    }

    pub fn get_root<'b>(&'b self, app: &'b App) -> RootRow {
        RootRow {
            sub_tasks: self.get_tasks(app, self.root)
        }
    }



}
