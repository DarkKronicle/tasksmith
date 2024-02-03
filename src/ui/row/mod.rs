use self::{task::TaskRow, text::TextRow};
use ratatui::{buffer::Buffer, layout::Rect};

use super::{style::SharedTheme, tasklist::{TableColumn, TaskWidgetState}};

pub mod text;
pub mod task;

// TODO: Bake this into config
pub const FOLD_OPEN: &str = " ";
pub const FOLD_CLOSE: &str = " ";

pub fn render_row(
    row: &RowEntry, 
    area: Rect, 
    buf: &mut Buffer, 
    state: &mut TaskWidgetState, 
    y: u16, 
    depth: u16, 
    theme: SharedTheme, 
    widths: &Vec<(TableColumn, u16, u16)>
) -> u16 {
    match row {
        RowEntry::Task(t) => {
            t.render(area, buf, state, y, depth, theme.clone(), widths)
        },
        RowEntry::Text(t) => {
            t.render(area, buf, state, y, depth, theme.clone(), widths)
        },
        _ => {
            0
        }
    }
}


#[derive(Debug)]
pub struct RootRow<'a> {
    pub sub_tasks: Vec<RowEntry<'a>>,
}


#[derive(Debug)]
pub enum RowEntry<'a> {
    Root(RootRow<'a>),
    Text(TextRow<'a>),
    Task(TaskRow<'a>),
}
