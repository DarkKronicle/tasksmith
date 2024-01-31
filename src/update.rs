use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;


pub fn on_key(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        _ => {},
    }
}
