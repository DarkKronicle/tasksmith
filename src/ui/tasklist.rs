use std::collections::HashMap;

use ratatui::{
    buffer::Buffer, 
    layout::{
        Constraint, 
        Direction, 
        Layout, 
        Rect}, 
    style::Style, 
    widgets::{
        Block, 
        Widget
    }
};
use uuid::Uuid;

use crate::{data::Task, tabs::list::List};

use super::{row::RowEntry, style::SharedTheme};

// Task widget will be able to do the following:
// - render tasks in a pretty way (colors)
// - be readable even if not interactable
// - customizable columns 
// - folds: dependencies, tags, projects, (hopefully anything)
//
// this full class is based heavily on https://github.com/ratatui-org/ratatui/blob/main/src/widgets/table/table.rs
#[derive(Debug)]
pub struct TaskListWidget<'a> {

    widths: Vec<Constraint>,

    style: Style,

    block: Option<Block<'a>>,

    rows: &'a [RowEntry],

    theme: SharedTheme,

}

impl<'a> TaskListWidget<'a> {

    pub fn new(rows: &'a [RowEntry], theme: SharedTheme) -> TaskListWidget<'a> {
        TaskListWidget {
            style: Default::default(),
            widths: vec![Constraint::Length(4), Constraint::Fill(40)],
            block: Default::default(),
            rows,
            theme: theme.clone()
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

}

#[derive(Debug, Clone, Copy)]
pub enum TableColumn {
    State,
    Description,
}


fn get_widths(widths: &Vec<Constraint>, columns: &[TableColumn], max_width: u16) -> Vec<(TableColumn, u16, u16)> {
    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(widths)
        .split(Rect::new(0, 0, max_width, 1));
    let mut column_iter = columns.iter();
    rects.iter().map(|c| (*column_iter.next().unwrap(), c.x, c.width)).collect()
}

impl TaskListWidget<'_> {

    pub fn render(self, area: Rect, buf: &mut Buffer, list: &List, task_map: &HashMap<Uuid, Task>) {
        buf.set_style(area, self.style);
        if let Some(b) = &self.block {
            b.clone().render(area, buf)
        }
        let widget_area = self.block.as_ref().map_or(area, |b| b.inner(area));

        if widget_area.is_empty() {
            return;
        }

        self.render_tasks(widget_area, buf, list, task_map);

    }

    fn render_tasks(&self, area: Rect, buf: &mut Buffer, list: &List, task_map: &HashMap<Uuid, Task>) {
        let mut y_offset = 0;

        let columns = vec![TableColumn::State, TableColumn::Description];
        let widths = get_widths(&self.widths, &columns, area.width);

        for (i, row) in self.rows.iter().enumerate() {
            if i < list.focus {
                continue;
            }
            let y_off = super::row::render_row(row, area, buf, super::row::RenderContext {
                y: y_offset,
                depth: 0,
                theme: self.theme.clone(),
                widths: &widths,
                list,
                index: i,
                task_map,
            });
            y_offset += y_off;
            if y_offset >= area.height {
                break;
            }
        };
    }

}
