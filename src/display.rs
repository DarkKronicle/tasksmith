use std::io;

use crate::task::Tasks;
use crate::ui::task::TaskWidget;

use ratatui::{prelude::*, widgets::*};

use color_eyre::Result;


fn ui(f: &mut Frame, tasks: Tasks) {
    let area = f.size();
    // let block = Block::default().title(block::Title::from("Hello world!").alignment(Alignment::Center));
    // f.render_widget(block, area);
    // let header = Row::new(vec!["Description", "Urg"]).bottom_margin(1).style(Style::new().bold());
    // let rows = tasks.tasks.iter().map(|task| Row::new(vec![task.description.clone(), task.urgency.to_string()]));
    // let widths = [
    //     Constraint::Percentage(95),
    //     Constraint::Percentage(5),
    // ];

    let widget = TaskWidget::new(&tasks);
    f.render_widget(widget, area);

}


pub fn display_table(tasks: Tasks) -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend, 
        TerminalOptions { viewport: Viewport::Inline(8) }
    )?;

    terminal.draw(|f| ui(f, tasks))?;

    crossterm::terminal::disable_raw_mode()?;
    // terminal.clear()?;
    Ok(())
}
