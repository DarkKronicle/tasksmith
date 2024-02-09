use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;


pub fn on_key(app: App, key_event: KeyEvent) -> App<'_> {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('k') => app.cursor(-1),
        KeyCode::Char('j') => app.cursor(1),
        KeyCode::Enter => {
            app.fold_entry()
        },
        _ => { app },
    }
}
