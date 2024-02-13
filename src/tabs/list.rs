use std::collections::{HashMap, VecDeque};
use color_eyre::Result;

use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{app::App, data::Task, event::Event, graph::{self, Separation}, ui::{row::RowEntry, tasklist::TaskListWidget}};



#[derive(Debug, Clone)]
pub struct List {
    row: RowEntry,
    pub cursor: usize,
    size: usize,
}

impl List {

    pub fn new(tasks: HashMap<Uuid, Task>) -> Self {
        let root = graph::into_root(tasks, Separation::Status);
        let row = RowEntry::Root(root);
        let size = row.len();
        List {
            row,
            cursor: 0,
            size
        }
    }

    pub fn draw(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        let list_component = TaskListWidget::new(&self.row, app.theme.clone());
        list_component.render(area, frame.buffer_mut(), self);
        Ok(())
    }

    pub fn is_folded(&self, index: usize) -> bool {
        false
    }

    fn cursor(&mut self, cursor: isize) {
        if cursor < 0 && cursor >= self.cursor as isize {
            // Clamp
            self.cursor = 0;
            return
        }
        if cursor == 0 {
            return
        }
        if cursor > 0 {
            self.cursor_forward(cursor as usize);
        } else {
            if self.cursor == 0 {
                return;
            }
            self.cursor_backward(cursor.abs() as usize);
        }
    }

    fn cursor_backward(&mut self, cursor: usize) {
        // This is a bit more complicated since we have to start at the cursor and work backwards
        // For right now, going to traverse the full thing then go backwards afterwards
        // Also only going to go to as far as the cursor
        // There's probably a better way to do this
        
        // We put usize here as well because why do the work twice!
        let mut traversal: VecDeque<(usize, &RowEntry)> = VecDeque::new();
        let mut first_traversal: VecDeque<&RowEntry> = self.row.sub_tasks().iter().rev().collect();
        let mut index = 0;
        while let Some(row) = first_traversal.pop_back() {
            traversal.push_back((index, row));
            if index >= self.cursor {
                break;
            }
            let sub = row.sub_tasks();
            if self.is_folded(index) {
                let size = row.len();
                // [)
                index += size;
                continue;
            }
            if !sub.is_empty() {
                first_traversal.extend(row.sub_tasks().iter().rev());
            }
            index += 1;
        }

        // We now have a stack representing each task and if they're folded or not
        for _ in 0..(cursor + 1) {
            let (idx, _) = traversal.pop_back().expect("missing tasks");
            self.cursor = idx;
        }
    }

    fn cursor_forward(&mut self, cursor: usize) {
        let mut index = 0;
        let mut togo = cursor.clone();
        // Will start top to bottom
        let mut traversal: VecDeque<&RowEntry> = self.row.sub_tasks().iter().rev().collect();
        while let Some(row) = traversal.pop_back() {
            if index > self.cursor {
                togo -= 1;
            }
            if togo == 0 {
                self.cursor = index;
                return;
            }

            if self.is_folded(index) {
                let size = row.len();
                // [)
                index += size;
                continue;
            }
            
            let sub = row.sub_tasks();
            if !sub.is_empty() {
                // We extend from the top to bottom so that we're in order with the render
                // traversal
                traversal.extend(row.sub_tasks().iter().rev());
            }
            index += 1;
        }
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::Key(k) => {
                match k.code {
                    KeyCode::Char('j') => {
                        self.cursor(1)
                    }
                    KeyCode::Char('k') => {
                        self.cursor(-1)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

}
