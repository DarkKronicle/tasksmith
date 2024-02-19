use std::collections::{HashMap, HashSet, VecDeque};
use color_eyre::Result;

use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{data::Task, event::Event, graph::{self, Separation}, ui::{row::RowEntry, style::SharedTheme, tasklist::TaskListWidget}};



#[derive(Debug, Clone)]
pub struct List {
    rows: Vec<RowEntry>,
    pub cursor: usize,
    pub focus: usize,
    last_size: Option<Rect>,
    folded: HashSet<usize>,
}

impl List {

    pub fn new(tasks: &HashMap<Uuid, Task>) -> Self {
        let rows = graph::get_tasks(tasks, Separation::Status);
        let mut idx = 0;
        // Fold default by default
        let mut hashset = HashSet::new();
        hashset.insert(idx);
        let mut list = List {
            rows,
            cursor: 0,
            focus: 0,
            folded: hashset,
            last_size: None,
        };
        // let flatten = list.flatten();
        // list.flatten = flatten;
        list
    }

    pub fn draw(&mut self, theme: SharedTheme, frame: &mut Frame, area: Rect, task_map: &HashMap<Uuid, Task>) -> Result<()> {
        let list_component = TaskListWidget::new(&self.rows, theme);
        self.last_size = Some(area);
        list_component.render(area, frame.buffer_mut(), self, task_map);
        Ok(())
    }

    pub fn is_folded(&self, index: usize) -> bool {
        self.folded.contains(&index)
    }

    fn focus(&mut self) {
        if let Some(area) = self.last_size {
            let height = area.height;
            let last_focus = self.focus;
            self.focus = self.keep_focus(height.into(), 7, self.cursor, last_focus);
        }
    }

    fn cursor(&mut self, change: isize) {
        if change < 0 && change >= self.cursor as isize {
            // Clamp
            self.cursor = 0;
            return
        }
        if change == 0 {
            return
        }
        if change > 0 {
            self.cursor = self.move_forward(self.cursor, change as usize);
        } else {
            if self.cursor == 0 {
                return;
            }
            self.cursor = self.move_backward(self.cursor, change.abs() as usize);
        }
    }

    fn keep_focus(&self, height: usize, padding: usize, cursor: usize, previous_focus: usize) -> usize {
        0
    }

    fn move_backward(&self, cursor: usize, change: usize) -> usize {
        0
    }

    fn move_forward(&self, cursor: usize, change: usize) -> usize {
        0
    }

    fn fold_row(&mut self, index: usize) {
        if !self.folded.remove(&index) {
            self.folded.insert(index);
        }
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::Key(k) => {
                match k.code {
                    KeyCode::Char('j') => {
                        self.cursor(1);
                        self.focus();
                    }
                    KeyCode::Char('k') => {
                        self.cursor(-1);
                        self.focus();
                    }
                    KeyCode::Enter => {
                        self.fold_row(self.cursor);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

}
