use color_eyre::eyre::Result;

use crate::{data::{get_tasks, Tasks}, ui::tree::TaskGraph};

pub struct App {
    pub should_quit: bool,
    pub task_graph: TaskGraph,
    pub tasks: Tasks,
}

impl App {

    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            task_graph: TaskGraph::empty(),
            tasks: Tasks::empty(),
        })
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        let tasks = get_tasks(Some("+PENDING"))?;
        let graph = TaskGraph::new(tasks);
        self.task_graph = graph;
        Ok(())
    }
    
}
