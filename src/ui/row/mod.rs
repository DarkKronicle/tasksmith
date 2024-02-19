use std::collections::HashMap;

use crate::{data::Task, tabs::list::List};

use self::{task::TaskRow, text::TextRow};
use ratatui::{buffer::Buffer, layout::Rect};
use uuid::Uuid;

use super::{style::SharedTheme, tasklist::TableColumn};

pub mod text;
pub mod task;

// TODO: Bake this into config
pub const FOLD_OPEN: &str = " ";
pub const FOLD_CLOSE: &str = " ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoldState {
    NoChildren,
    Folded,
    Open,
}

pub fn render_row(
    row: &RowEntry, 
    area: Rect, 
    buf: &mut Buffer, 
    context: RenderContext
) -> u16 {
    match row {
        RowEntry::Task(t) => {
            let offset = t.render(area, buf, context);
            offset
        },
        RowEntry::Text(t) => {
            let offset = t.render(area, buf, context);
            offset
        },
        _ => {
            context.y
        }
    }
}


#[derive(Debug, Clone)]
pub enum RowEntry {
    Text(TextRow),
    Task(TaskRow),
}


pub struct RenderContext<'a> {
    pub y: u16, 
    pub depth: u16, 
    pub theme: SharedTheme, 
    pub widths: &'a Vec<(TableColumn, u16, u16)>,
    pub list: &'a List,
    pub index: usize,
    pub task_map: &'a HashMap<Uuid, Task>,
}
