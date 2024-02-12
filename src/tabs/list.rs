use std::{collections::HashMap};
use color_eyre::Result;

use ratatui::{layout::Rect, Frame};
use uuid::Uuid;

use crate::{app::App, data::Task, event::Event, graph::{self, Separation}, ui::{row::RowEntry, tasklist::TaskListWidget}};



#[derive(Debug, Clone)]
pub struct List {
    row: RowEntry
}

impl List {

    pub fn new(tasks: HashMap<Uuid, Task>) -> Self {
        let root = graph::into_root(tasks, Separation::Status);
        List {
            row: RowEntry::Root(root)
        }
    }

    pub fn draw(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        let list_component = TaskListWidget::new(&self.row, app.theme.clone());
        list_component.render(area, frame.buffer_mut());
        Ok(())
    }

    pub fn event(&mut self, event: Event) {

    }

}
