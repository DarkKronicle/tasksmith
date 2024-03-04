use std::collections::HashMap;

use crate::{data::Task, tabs::list::List};

use self::{task::TaskRow, text::TextRow};
use ratatui::{buffer::Buffer, layout::Rect};
use uuid::Uuid;

use super::{style::SharedTheme, tasklist::TableColumn};

pub mod text;
pub mod task;

// TODO: Bake this into config
pub const FOLD_OPEN: &str = " ";
pub const FOLD_CLOSE: &str = " ";

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
            t.render(area, buf, context)
        },
        RowEntry::Text(t) => {
            t.render(area, buf, context)
        },
    }
}

#[derive(Debug, Clone)]
pub enum RowEntry {
    Text(TextRow),
    Task(TaskRow),
}

impl RowEntry {

    pub fn fold_state(&self) -> FoldState {
        match self {
            RowEntry::Task(t) => t.fold_state.clone(),
            RowEntry::Text(t) => t.fold_state.clone(),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            RowEntry::Task(t) => t.idx,
            RowEntry::Text(t) => t.idx,
        }
    }
    
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
