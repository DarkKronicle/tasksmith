use std::collections::HashMap;

use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::{data::{get_tasks, Task}, ui::{row::{RootRow, RowEntry}, style::SharedTheme, taskgraph::TaskGraph, tasklist::TaskWidgetState}};

#[derive(Clone, Debug)]
pub struct App<'a> {
    pub should_quit: bool,
    pub list_root: RowEntry<'a>,
    pub theme: SharedTheme,
    pub tasklist_state: TaskWidgetState,
}

impl<'a> App<'a> {

    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            theme: SharedTheme::default(),
            tasklist_state: TaskWidgetState::default(),
            list_root: RowEntry::Root(RootRow::default()),
        })
    }

    pub fn tick(&self) {}

    pub fn quit(self) -> Self {
        App { 
            should_quit: true, 
            ..self
        }
    }

    pub fn cursor(self, val: isize) -> Self {
        let state = self.tasklist_state.cursor(val);
        App { 
            tasklist_state: state,
            ..self
        }
    }

    pub fn fold_entry(self) -> Self {
        let mut remove: Option<bool> = None;
        if let Some(c) = self.tasklist_state.cursor {
            if !self.tasklist_state.folded.contains(&c) {
                if let Some(row) = self.list_root.get(c){ 
                    if row.has_children() {
                        remove = Some(false);
                    }
                }
            } else {
                remove = Some(true);
            }
        };
        let state = if let Some(rem) = remove {
            self.tasklist_state.fold_entry(rem)
        } else {
            self.tasklist_state
        };
        App { 
            tasklist_state: state,
            ..self
        }
    }

    pub fn refresh_tasks(self, graph: &'a TaskGraph, tasks: &'a HashMap<Uuid, Task>) -> Result<Self> {
        let list_root = TaskGraph::get_root(graph, tasks);
        Ok(App {
            list_root: RowEntry::Root(list_root),
            ..self
        })
    }
    
}
