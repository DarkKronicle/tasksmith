use std::{alloc::Layout, cell::RefCell, collections::HashMap, io, rc::Rc, sync::atomic::{AtomicBool, Ordering}};

use color_eyre::eyre::Result;
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use uuid::Uuid;

use crate::{data::{get_tasks, Task}, event::Event, tabs::list::List, ui::style::SharedTheme};

#[derive(Debug)]
pub struct App {
    pub should_quit: AtomicBool,
    pub theme: SharedTheme,
    pub list: List,
    pub tasks: HashMap<Uuid, Task>,
}

impl App {

    pub fn new() -> Result<Self> {
        let task_map = get_tasks()?;
        let list = List::new(task_map.clone());
        Ok(Self {
            should_quit: false.into(),
            theme: SharedTheme::default(),
            list,
            tasks: task_map,
        })
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self)  {
        self.should_quit.swap(true, Ordering::Relaxed);
    }

    pub fn draw(&self, frame: &mut Frame) -> Result<()> {
        let fsize = frame.size();
        self.list.draw(self, frame, fsize)?;
        Ok(())
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::Key(k) => {
                self.quit()
            },
            _ => {}
        }
        
    }

}
