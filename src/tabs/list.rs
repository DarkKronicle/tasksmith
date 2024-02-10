use std::{cell::RefCell, collections::HashMap, rc::Rc};
use color_eyre::Result;

use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{app::App, data::Task, graph, ui::{row::{RootRow, RowEntry}, taskgraph::TaskGraph, tasklist::TaskListWidget}};



#[derive(Debug, Clone)]
pub struct List {
    row: RowEntry
}

impl List {

    pub fn new(tasks: HashMap<Uuid, Task>) -> Self {
        let root = graph::into_root(tasks);
        List {
            row: RowEntry::Root(root)
        }
    }

    pub fn draw(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        let list_component = TaskListWidget::new(&self.row, app.theme.clone());
        list_component.render(area, frame.buffer_mut());
        Ok(())
    }

}
