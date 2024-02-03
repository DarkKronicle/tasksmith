use crate::app::App;
use crate::ui::tasklist::TaskListWidget;

use color_eyre::Result;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;


pub fn draw(f: &mut Frame, app: &mut App) -> Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(f.size());
    let root = app.task_graph.get_root(&app.tasks);
    let task_widget = TaskListWidget::new(&root, &app).block(
        Block::default()
        .style(app.theme.border())
    );
    let mut state = app.tasklist_state.clone();
    f.render_stateful_widget(task_widget, chunks[1], &mut state);
    app.tasklist_state = state;
    
    let filter_widget = Block::default()
        .borders(Borders::RIGHT)
        .style(app.theme.border());
    f.render_widget(filter_widget, chunks[0]);
    Ok(())
}
