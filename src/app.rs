use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{data::{get_tasks, Task}, event::Event, tabs::list::List, ui::style::SharedTheme};

#[derive(Debug)]
pub struct App {
    pub should_quit: AtomicBool,
    pub theme: SharedTheme,
    pub list: List,
    pub tasks: HashMap<Uuid, Task>,
    last_size: Option<Rect>
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
            last_size: None
        })
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self)  {
        self.should_quit.swap(true, Ordering::Relaxed);
    }

    pub fn draw(&mut self, frame: &mut Frame) -> Result<()> {
        let fsize = frame.size();
        self.list.draw(self.theme.clone(), frame, fsize)?;
        Ok(())
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::Key(k) => {
                match k.code {
                    KeyCode::Char('c') => {
                        if KeyModifiers::CONTROL == k.modifiers {
                            self.quit()
                        }
                    },
                    KeyCode::Char('q') => self.quit(),
                    _ => {}
                }
            },
            _ => {}
        }
        self.list.event(event);
        
    }

}
