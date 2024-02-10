use self::{task::TaskRow, text::TextRow};
use ratatui::{buffer::Buffer, layout::Rect};

use super::{style::SharedTheme, tasklist::TableColumn};

pub mod text;
pub mod task;

// TODO: Bake this into config
pub const FOLD_OPEN: &str = " ";
pub const FOLD_CLOSE: &str = " ";

pub fn render_row(
    row: &RowEntry, 
    area: Rect, 
    buf: &mut Buffer, 
    context: RenderContext
) -> (usize, u16) {
    match row {
        RowEntry::Task(t) => {
            let (idx, offset) = t.render(area, buf, context);
            (idx, offset)
        },
        RowEntry::Text(t) => {
            let (idx, offset) = t.render(area, buf, context);
            (idx, offset)
        },
        _ => {
            (context.index, context.y)
        }
    }
}


#[derive(Debug, Clone)]
pub struct RootRow {
    pub sub_tasks: Vec<RowEntry>,
}

impl Default for RootRow {
    fn default() -> RootRow {
        RootRow { sub_tasks: vec![] }
    }
}


#[derive(Debug, Clone)]
pub enum RowEntry {
    Root(RootRow),
    Text(TextRow),
    Task(TaskRow),
}


impl RowEntry {

    pub fn sub_tasks(&self) -> &Vec<RowEntry> {
        match self {
            Self::Root(r) => &r.sub_tasks,
            Self::Text(r) => &r.sub_tasks,
            Self::Task(r) => &r.sub_tasks,
        }
    }

    pub fn has_children(&self) -> bool {
        self.sub_tasks().len() > 0
    }
    
    pub fn len(&self) -> usize {
        let count: usize = self.sub_tasks().iter().map(|t| t.len()).sum();
        // Got to count this one
        count + 1
    }

    pub fn get(&self, index: usize) -> Option<&RowEntry> {
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
    
    pub fn get_from_root(row: &RootRow, index: usize) -> Option<&RowEntry> {
        if index == 0 {
            return None
        }
        let mut idx = 0;
        for t in row.sub_tasks.iter() {
            let len = t.len();
            idx += 1;
            if idx + len > index {
                return t.get(index - idx);
            }
            idx += len - 1;
        }
        None
    }

    pub fn add_task(&mut self, row: RowEntry) {
        match self {
            Self::Root(r) => r.sub_tasks.push(row),
            Self::Text(r) => r.sub_tasks.push(row),
            Self::Task(r) => r.sub_tasks.push(row),
        }
    }

}


pub struct RenderContext<'a> {
    pub y: u16, 
    pub depth: u16, 
    pub theme: SharedTheme, 
    pub widths: &'a Vec<(TableColumn, u16, u16)>,
    pub index: usize,
}
