use std::collections::{HashMap, HashSet, VecDeque};
use color_eyre::Result;

use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{app::App, data::Task, event::Event, graph::{self, Separation}, ui::{row::RowEntry, style::SharedTheme, tasklist::TaskListWidget}};



#[derive(Debug, Clone)]
pub struct List {
    row: RowEntry,
    pub cursor: usize,
    start_task: usize,
    size: usize,
    pub focus: usize,
    last_size: Option<Rect>,
    folded: HashSet<usize>,
}

impl List {

    pub fn new(tasks: HashMap<Uuid, Task>) -> Self {
        let root = graph::into_root(tasks, Separation::Status);
        let subs = root.sub_tasks.len();
        let mut idx = 0;
        // Fold default by default
        // TODO: make this a bit more abstract
        for (i, sub) in root.sub_tasks.iter().enumerate() {
            if i == (subs - 1) {
                break;
            } else {
                idx += sub.len();
            }
        }
        let row = RowEntry::Root(root);
        let size = row.len();
        let mut hashset = HashSet::new();
        hashset.insert(idx);
        List {
            row,
            cursor: 0,
            focus: 0,
            start_task: 0,
            size,
            folded: hashset,
            last_size: None,
        }
    }

    pub fn draw(&mut self, theme: SharedTheme, frame: &mut Frame, area: Rect) -> Result<()> {
        let list_component = TaskListWidget::new(&self.row, theme);
        self.last_size = Some(area);
        list_component.render(area, frame.buffer_mut(), self);
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
        let mut lines: Vec<usize> = Vec::new();
        let mut traversal: VecDeque<&RowEntry> = self.row.sub_tasks().iter().rev().collect();
        let mut index = 0;
        let mut cursor_index = None;
        let mut previous_index = None;
        while let Some(row) = traversal.pop_back() {
            if index == cursor {
                cursor_index = Some(index);
            }
            if index == previous_focus {
                previous_index = Some(index);
            }
            // if cursor_index.is_some() && previous_index.is_some() {
            //     break;
            // }

            lines.push(index);
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
        };
        if let Some(c) = cursor_index {
            if previous_index.is_none() {
                // Hmm, previous index doesn't exist, so we just set it to cursor - previous_focus
                if c <= padding {
                    return *lines.get(0).unwrap();
                }
                return *lines.get(c - padding).unwrap();
            };
        } else {
            return 0;
        };
        // These are essentially line numbers
        let cursor_index = cursor_index.unwrap();
        let previous_index = previous_index.unwrap();
        // Bottom
        if cursor_index < previous_index || cursor_index - previous_index < padding {
            if padding > cursor_index {
                if lines.is_empty() {
                    return 0;
                }
                return *lines.first().unwrap();
            }
            return *lines.get(cursor_index - padding).unwrap();
        }
        // Top
        let relative_position = cursor_index - previous_index;
        if relative_position > height || height - relative_position < padding {
            if lines.len() < cursor_index + padding {
                return *lines.last().unwrap();
            }
            return *lines.get(cursor_index + padding - height).unwrap();
        }
        previous_focus
    }

    fn move_backward(&self, cursor: usize, change: usize) -> usize {
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
            if index >= cursor {
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

        // TODO: remove this variable, and just directly return it.
        let mut end = 0;
        for _ in 0..(change + 1) {
            let (idx, _) = traversal.pop_back().expect("missing tasks");
            
            end = idx;
        }
        end
    }

    fn move_forward(&self, cursor: usize, change: usize) -> usize {
        let mut index = 0;
        let mut togo = change.clone();
        // Will start top to bottom
        let mut traversal: VecDeque<&RowEntry> = self.row.sub_tasks().iter().rev().collect();
        while let Some(row) = traversal.pop_back() {
            if index > cursor {
                togo -= 1;
            }
            if togo == 0 {
                return index;
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
        };
        index - 1
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
