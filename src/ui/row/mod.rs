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
    context: RenderContext
) -> (usize, u16) {
    match row {
        RowEntry::Task(t) => {
            let (idx, offset) = t.render(area, buf, state, context);
            (idx, offset)
        },
        RowEntry::Text(t) => {
            let (idx, offset) = t.render(area, buf, state, context);
            (idx, offset)
        },
        _ => {
            (context.index, context.y)
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


impl<'a> RowEntry<'a> {

    pub fn sub_tasks(&self) -> &Vec<RowEntry<'a>> {
        match self {
            Self::Root(r) => &r.sub_tasks,
            Self::Text(r) => &r.sub_tasks,
            Self::Task(r) => &r.sub_tasks,
        }
    }

    pub fn folded(&self) -> bool {
        match self {
            Self::Root(_) => false,
            Self::Task(r) => r.folded,
            Self::Text(r) => r.folded,
        }
    }
    
    pub fn len(&self) -> usize {
        let count: usize = self.sub_tasks().iter().map(|t| t.len()).sum();
        // Got to count this one
        count + 1
    }

    pub fn get(&self, index: usize) -> Option<&RowEntry<'a>> {
        if index == 0 {
            return Some(self)
        }
        let mut idx = 0;
        for t in self.sub_tasks().iter() {
            let len = t.len();
            idx += 1;
            if idx + len > index {
                return t.get(index - idx);
            }
            idx += len - 1;
        }
        None
    }

}


pub struct RenderContext<'a> {
    pub y: u16, 
    pub depth: u16, 
    pub theme: SharedTheme, 
    pub widths: &'a Vec<(TableColumn, u16, u16)>,
    pub index: usize,
}
