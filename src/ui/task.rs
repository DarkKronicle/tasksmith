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
        StatefulWidget, 
        Widget
    }
};

use crate::data::{Tasks, Task};

use super::tree::{RootRow, RowEntry, TaskGraph};

// Task widget will be able to do the following:
// - render tasks in a pretty way (colors)
// - be readable even if not interactable
// - customizable columns 
// - folds: dependencies, tags, projects, (hopefully anything)
//
// this full class is based heavily on https://github.com/ratatui-org/ratatui/blob/main/src/widgets/table/table.rs
#[derive(Debug)]
pub struct TaskWidget<'a> {

    widths: Vec<Constraint>,

    style: Style,

    block: Option<Block<'a>>,

    root: RootRow<'a>,

}

pub struct TaskWidgetState<'a> {
    pub selected: Option<&'a Task>,
}

impl Default for TaskWidgetState<'_> {

    fn default() -> Self {
        Self { selected: None }
    }
    
}


impl TaskWidget<'_> {

    pub fn new<'a>(tasks: &'a TaskGraph) -> TaskWidget<'a> {
        TaskWidget {
            style: Default::default(),
            widths: Default::default(),
            block: Default::default(),
            root: tasks.get_root()
        }
    }

}


impl Widget for TaskWidget<'_> {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TaskWidgetState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }

}

impl Widget for &TaskWidget<'_> {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TaskWidgetState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }

}


impl<'a> StatefulWidget for TaskWidget<'a> {
    type State = TaskWidgetState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }

}


impl<'a> StatefulWidget for &TaskWidget<'a> {
    type State = TaskWidgetState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        if let Some(b) = &self.block {
            b.clone().render(area, buf)
        }
        let widget_area = self.block.as_ref().map_or(area, |b| b.inner(area));

        if widget_area.is_empty() {
            return;
        }

        let (header_area, rows_area) = self.layout(widget_area);

        self.render_tasks(rows_area, buf, state);

    }

}

impl TaskWidget<'_> {

    fn layout(&self, area: Rect) -> (Rect, Rect) {
        let layout = Layout::new(Direction::Vertical, [
            Constraint::Length(1),
            Constraint::Min(0),
        ]).split(area);
        (layout[0], layout[1])
    }

    fn render_tasks(&self, area: Rect, buf: &mut Buffer, state: &mut TaskWidgetState) {
        if self.root.sub_tasks.is_empty() {
            return;
        }

        let mut y_offset = 0;

        for (i, row) in self.root.sub_tasks.iter().enumerate() {
            match row {
                RowEntry::Task(trow) => {
                    y_offset += trow.render(area, buf, state, 0, y_offset, 0)
                },
                RowEntry::Root(_) => {},
            }
            if y_offset >= area.height {
                break;
            }
        };
    }

}
