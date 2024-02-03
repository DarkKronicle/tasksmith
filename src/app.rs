use std::collections::HashMap;

use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::{data::{get_tasks, Task}, ui::{row::RootRow, style::SharedTheme, taskgraph::TaskGraph, tasklist::TaskWidgetState}};

#[derive(Clone, Debug)]
pub struct App<'a> {
    pub should_quit: bool,
    pub list_root: RootRow<'a>,
    pub task_graph: TaskGraph,
    pub tasks: HashMap<Uuid, Task>,
    pub theme: SharedTheme,
    pub tasklist_state: TaskWidgetState,
}

impl<'a> App<'a> {

    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            task_graph: TaskGraph::empty(),
            tasks: HashMap::default(),
            theme: SharedTheme::default(),
            tasklist_state: TaskWidgetState::default(),
            list_root: RootRow::default(),
        })
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = get_tasks(None)?;
        self.task_graph = TaskGraph::new(&self.tasks);
        Ok(())
    }

    pub fn update_list_root(&'a mut self) {
        let graph = &self.task_graph;
        let root = graph.get_root(&self.tasks);
        self.list_root = root;
    }
    
}
