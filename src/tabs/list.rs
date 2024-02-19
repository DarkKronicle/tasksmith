use std::{cmp::Ordering, collections::{HashMap, HashSet, VecDeque}};
use color_eyre::Result;

use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{data::Task, event::Event, ui::{row::{task::TaskRow, FoldState, RowEntry}, style::SharedTheme, tasklist::TaskListWidget}, util::{self, graph::{Idable, Node}}};



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
        let mut hashset = HashSet::new();
        let rows = get_tasks(tasks, Separation::Status, &hashset);
        let mut idx = 0;
        // Fold default by default
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
            self.focus = List::keep_focus(height.into(), 7, self.cursor, last_focus, self.rows.len());
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
            self.cursor = self.move_backward(self.cursor, change.unsigned_abs());
        }
    }

    fn keep_focus(height: usize, padding: usize, cursor: usize, previous_focus: usize, max_len: usize) -> usize {
        // Lower bound
        if cursor <= padding {
            return 0
        }
        // Add done to upper to make it symmetrical
        let padding_lower = padding;
        let padding_upper = padding + 1;

        let previous_high = previous_focus + height;
        if previous_high - padding_upper >= cursor && previous_focus + padding_lower <= cursor {
            // We're still in focus
            return previous_focus
        }
        if cursor <= previous_focus + padding_lower {
            // Moving down
            return cursor - padding

        }
        let target = cursor + padding_upper - height;
        if target > max_len {
            max_len
        } else {
            target
        }
    }

    fn move_backward(&self, cursor: usize, change: usize) -> usize {
        if change >= cursor {
            0
        } else {
            cursor - change
        }
    }

    fn move_forward(&self, cursor: usize, change: usize) -> usize {
        if cursor + change >= self.rows.len() {
            self.rows.len() - 1
        } else {
            cursor + change
        }
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


pub enum Separation {
    None,
    Status,
}

pub fn get_tasks(tasks: &HashMap<Uuid, Task>, separation: Separation, folded: &HashSet<usize>) -> Vec<RowEntry> {
    let mut nodes = util::graph::graph_nodes(tasks);

    sort_tasks(&mut nodes, tasks);

    // Reverse because we go back to front
    nodes.reverse();

    let mut traverse: VecDeque<_> = nodes.into_iter().collect();
    let mut depth: VecDeque<usize> = VecDeque::new();
    let mut rows = Vec::new();
    let mut idx = 0;

    while let Some(node) = traverse.pop_back() {
        let fold_state = if node.child_len() == 0 {
            FoldState::NoChildren
        } else if folded.contains(&idx) {
            FoldState::Folded
        } else {
            FoldState::Open
        };
        let row = RowEntry::Task(
            TaskRow { 
                task: node.get_id(), 
                depth: depth.len(), 
                fold_state: fold_state.clone()
            }
        );
        rows.push(row);
        let d_opt = depth.pop_back().map(|v| if v > 0 { v - 1 } else { v });
        if !node.sub.is_empty() && fold_state != FoldState::Folded {
            // Children
            if let Some(d) = d_opt {
                depth.push_back(d);
            }
            depth.push_back(node.sub.len());
            traverse.extend(node.sub);
        } else {
            // No Children/folded
            if let Some(d) = d_opt {
                if d > 0 {
                    depth.push_back(d);
                }
            }
            while depth.back().map_or_else(|| { false }, |d| {*d == 0}) {
                depth.pop_back();
            }
            idx += node.child_len();
        }
        idx += 1;
    }

    rows
}


pub fn sort_tasks(rows: &mut [Node], tasks: &HashMap<Uuid, Task>) {
    rows.sort_by(|a, b| {
        let a = tasks.get(a.get_id_ref()).unwrap();
        let b = tasks.get(b.get_id_ref()).unwrap();
        if a.status == b.status {
            let cmp = b.urgency.partial_cmp(&a.urgency).expect("Invalid urgency");
            return match cmp {
                Ordering::Equal => {
                    b.description.cmp(&a.description)
                },
                _ => cmp
            }
        }
        a.status.partial_cmp(&b.status).expect("Invalid status")
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_focus_up() {
        let height = 30;
        let padding = 5;
        let max = 60;
        //                                        cursor/previous
        assert_eq!(List::keep_focus(height, padding, 0, 0, max), 0);
        assert_eq!(List::keep_focus(height, padding, 30, 0, max), 6);
        assert_eq!(List::keep_focus(height, padding, 31, 0, max), 7);
        assert_eq!(List::keep_focus(height, padding, 41, 0, max), 17);
        assert_eq!(List::keep_focus(height, padding, 41, 30, max), 30);
    }

    #[test]
    fn keep_focus_down() {
        let height = 30;
        let padding = 5;
        let max = 60;
        //                                        cursor/previous
        assert_eq!(List::keep_focus(height, padding, 0, 30, max), 0);
        assert_eq!(List::keep_focus(height, padding, 0, 1, max), 0);
        assert_eq!(List::keep_focus(height, padding, 25, 30, max), 20);
        assert_eq!(List::keep_focus(height, padding, 25, 26, max), 20);
        assert_eq!(List::keep_focus(height, padding, 25, 40, max), 20);
        assert_eq!(List::keep_focus(height, padding, 25, 24, max), 20);
    }

}
