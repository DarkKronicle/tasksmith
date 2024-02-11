use std::{io::{self, Write}, panic, sync::atomic::Ordering};

use color_eyre::eyre::Result;
use crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};

mod data;
mod graph;
mod ui;
mod app;
mod event;
mod tabs;


fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = terminal_enter(std::io::stdout())?;
    run(&mut terminal)?;
    terminal_reset()?;
    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) ->Result<()> {

    let events = EventHandler::new(250);
    let mut app = app::App::new()?;
    
    {
        while !app.should_quit.load(Ordering::Relaxed) {
            draw(terminal, &mut app)?;

            let event = events.next()?;
            match event {
                Event::Tick => {},
                _ => {
                    app.event(event)
                },
            }
        }
    }
    Ok(())
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut app::App) -> Result<()> {
    terminal.draw(|f| app.draw(f).expect("failed to draw"))?;
    Ok(())
}

fn terminal_enter<W: Write>(buf: W) -> Result<Terminal<CrosstermBackend<W>>> {
    let mut term = Terminal::new(CrosstermBackend::new(buf))?;
    terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stderr(),
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        terminal_reset().expect("failed to reset the terminal");
        panic_hook(panic);
    }));

    term.hide_cursor()?;
    term.clear()?;

    Ok(term)
}


fn terminal_reset() -> Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stderr(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    Ok(())
}
