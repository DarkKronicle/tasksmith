use std::{cell::RefCell, collections::{HashMap, VecDeque}, ops::Index};

use crate::data::{Task, Tasks};
use color_eyre::Result;
use petgraph::{data::Build, graph::{NodeIndex, UnGraph}, Direction, Graph};
use ratatui::{buffer::Buffer, layout::Rect, text::Text};
use uuid::Uuid;
use petgraph::visit::{Dfs, Walker};

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
    fn compare(&self, other: &dyn Row, sorttype: SortType) -> std::cmp::Ordering {
        todo!()
    }

    fn action(&self, action: RowAction) -> Result<()> {
        todo!()
    }
}

impl<'a> Row for TaskRow<'a> {
    fn compare(&self, other: &dyn Row, sorttype: SortType) -> std::cmp::Ordering {
        todo!()
    }

    fn action(&self, action: RowAction) -> Result<()> {
        todo!()
    }
}

impl<'a> TaskRow<'a> {
    fn from(task: &'a Task) -> Self {
        TaskRow {
            task,
            sub_tasks: vec![].into()
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
    graph: Graph<Option<Task>, ()>,
    node_map: HashMap<Uuid, NodeIndex>,
    root: NodeIndex,
    tasks: Tasks,
}

// TODO: When I'm good at rust look at this again, I think there's a better way to do this
impl TaskGraph {

    pub fn empty() -> Self {
        let mut graph = Graph::new();
        let root = graph.add_node(None);
        Self {
            graph,
            node_map: HashMap::new(),
            root,
            tasks: Tasks::empty()
        }
    }

    pub fn new(tasks: Tasks) -> Self {
        let mut graph = Graph::<Option<Task>, ()>::new();
        let mut node_map = HashMap::new();
        let root = graph.add_node(None);

        for task in &tasks.tasks.clone() {
            let index = graph.add_node(Some(task.clone()));
            node_map.insert(task.uuid, index);
        }

        for task in &tasks.tasks {
            if let Some(parent_uuid) = &task.sub_of {
                if let Some(&parent_index) = node_map.get(parent_uuid) {
                    let child_index = node_map[&task.uuid];
                    graph.add_edge(parent_index, child_index, ());
                }
            } else {
                let child_index = node_map[&task.uuid];
                graph.add_edge(root, child_index, ());
            }
        }

        TaskGraph {
            graph,
            node_map,
            root,
            tasks
        }
    }

    pub fn get_tasks(self: &Self, node: NodeIndex) -> Vec<RowEntry> {
        let mut tasks = vec![];
        let neighbors: Vec<_> = self.graph.neighbors_directed(node, Direction::Outgoing).collect();

        for n in neighbors {
            tasks.push(RowEntry::Task(TaskRow {
                task: self.graph.node_weight(n).unwrap().as_ref().unwrap(),
                sub_tasks: self.get_tasks(n)
            }));
        };
        tasks
    }

    pub fn get_root(&self) -> RootRow {
        RootRow {
            sub_tasks: self.get_tasks(self.root)
        }
    }



}
