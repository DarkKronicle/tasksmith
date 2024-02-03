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

use crate::app::App;

use super::{row::RootRow, style::SharedTheme, taskgraph::TaskGraph};

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

    root: RootRow<'a>,

    theme: SharedTheme,

}

#[derive(Default, Clone, Debug)]
pub struct TaskWidgetState {
    pub cursor: Option<usize>,
}

impl TaskWidgetState {

    pub fn cursor(self: &mut Self, amount: isize) {
        if let Some(c) = self.cursor {
            let new_c: isize = (c as isize) + amount;
            if new_c < 0 {
                self.cursor = Some(0);
            } else {
                self.cursor = Some(new_c as usize);
            }
            return;
        }
        if amount > 0 {
            self.cursor = Some(amount as usize);
        } else {
            self.cursor = Some(0);
        }
    }

}

impl<'a> TaskListWidget<'a> {

    pub fn new(tasks: &'a TaskGraph, app: &'a App) -> TaskListWidget<'a> {
        TaskListWidget {
            style: Default::default(),
            widths: vec![Constraint::Length(4), Constraint::Fill(40)],
            block: Default::default(),
            root: tasks.get_root(app),
            theme: app.theme.clone()
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

}


impl Widget for TaskListWidget<'_> {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TaskWidgetState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }

}

impl Widget for &TaskListWidget<'_> {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TaskWidgetState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }

}


impl<'a> StatefulWidget for TaskListWidget<'a> {
    type State = TaskWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }

}


impl<'a> StatefulWidget for &TaskListWidget<'a> {
    type State = TaskWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        if let Some(b) = &self.block {
            b.clone().render(area, buf)
        }
        let widget_area = self.block.as_ref().map_or(area, |b| b.inner(area));

        if widget_area.is_empty() {
            return;
        }

        self.render_tasks(widget_area, buf, state);

    }

}

#[derive(Debug, Clone, Copy)]
pub enum TableColumn {
    State,
    Description,
}


fn get_widths(widths: &Vec<Constraint>, columns: &Vec<TableColumn>, max_width: u16) -> Vec<(TableColumn, u16, u16)> {
    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(widths)
        .split(Rect::new(0, 0, max_width, 1));
    let mut column_iter = columns.iter();
    rects.iter().map(|c| (*column_iter.next().unwrap(), c.x, c.width)).collect()
}

impl TaskListWidget<'_> {

    fn render_tasks(&self, area: Rect, buf: &mut Buffer, state: &mut TaskWidgetState) {
        if self.root.sub_tasks.is_empty() {
            return;
        }

        let mut y_offset = 0;

        let columns = vec![TableColumn::State, TableColumn::Description];
        let widths = get_widths(&self.widths, &columns, area.width);
        let mut idx = 0;

        for (i, row) in self.root.sub_tasks.iter().enumerate() {
            if state.cursor.is_none() {
                state.cursor = Some(i);
            }
            let (index, y_off) = super::row::render_row(row, area, buf, state, y_offset, 0, self.theme.clone(), &widths, idx);
            y_offset += y_off;
            idx = index;
            if y_offset >= area.height {
                break;
            }
        };
    }

}
