use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;


pub fn on_key(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('k') => app.tasklist_state.cursor(-1),
        KeyCode::Char('j') => app.tasklist_state.cursor(1),
        KeyCode::Enter => {
            // app.tasklist_state.fold(&app.list_root)
        },
        _ => {},
    }
}
