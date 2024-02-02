use crate::app::App;
use crate::ui::task::TaskWidget;

use color_eyre::Result;
use ratatui::Frame;


pub fn draw(f: &mut Frame, app: &mut App) -> Result<()> {
    let area = f.size();
    let task_widget = TaskWidget::new(&app.task_graph, app);
    f.render_widget(task_widget, area);
    Ok(())
}
