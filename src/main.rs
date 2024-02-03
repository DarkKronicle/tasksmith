use color_eyre::eyre::Result;
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;

mod display;
mod data;
mod ui;
mod tui;
mod app;
mod event;
mod update;


fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = app::App::new()?;
    app.refresh_tasks()?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let events = EventHandler::new(250);
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => {},
            Event::Key(key) => update::on_key(&mut app, key),
            Event::Mouse(_) => {},
            Event::Resize(_, _) => {},
        }
    }

    tui.exit()?;
    Ok(())
}
