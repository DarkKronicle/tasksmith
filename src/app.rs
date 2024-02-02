use std::{collections::HashMap};

use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::{data::{get_tasks, Task}, ui::tree::TaskGraph};

pub struct App {
    pub should_quit: bool,
    pub task_graph: TaskGraph,
    pub tasks: HashMap<Uuid, Task>,
}

impl App {

    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            task_graph: TaskGraph::empty(),
            tasks: HashMap::default(),
        })
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = get_tasks(Some("+PENDING"))?;
        self.task_graph = TaskGraph::new(self);
        Ok(())
    }
    
}
