use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Quit,
    NavigateUp,
    NavigateDown,
    Enter,
    Back,
    None,
}

pub fn poll_event(timeout: Duration) -> Result<Event> {
    if event::poll(timeout)? {
        if let CrosstermEvent::Key(key) = event::read()? {
            return Ok(handle_key_event(key));
        }
    }
    Ok(Event::None)
}

fn handle_key_event(key: KeyEvent) -> Event {
    if key.kind != KeyEventKind::Press {
        return Event::None;
    }

    match key.code {
        KeyCode::Char('q') => Event::Quit,
        KeyCode::Esc => Event::Back,
        KeyCode::Enter => Event::Enter,
        KeyCode::Down | KeyCode::Char('j') => Event::NavigateDown,
        KeyCode::Up | KeyCode::Char('k') => Event::NavigateUp,
        _ => Event::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventState, KeyModifiers};

    fn make_key_event(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_handle_key_event_quit() {
        let event = handle_key_event(make_key_event(KeyCode::Char('q'), KeyEventKind::Press));
        assert_eq!(event, Event::Quit);
    }

    #[test]
    fn test_handle_key_event_back() {
        let event = handle_key_event(make_key_event(KeyCode::Esc, KeyEventKind::Press));
        assert_eq!(event, Event::Back);
    }

    #[test]
    fn test_handle_key_event_enter() {
        let event = handle_key_event(make_key_event(KeyCode::Enter, KeyEventKind::Press));
        assert_eq!(event, Event::Enter);
    }

    #[test]
    fn test_handle_key_event_navigate_down() {
        let event = handle_key_event(make_key_event(KeyCode::Down, KeyEventKind::Press));
        assert_eq!(event, Event::NavigateDown);

        let event = handle_key_event(make_key_event(KeyCode::Char('j'), KeyEventKind::Press));
        assert_eq!(event, Event::NavigateDown);
    }

    #[test]
    fn test_handle_key_event_navigate_up() {
        let event = handle_key_event(make_key_event(KeyCode::Up, KeyEventKind::Press));
        assert_eq!(event, Event::NavigateUp);

        let event = handle_key_event(make_key_event(KeyCode::Char('k'), KeyEventKind::Press));
        assert_eq!(event, Event::NavigateUp);
    }

    #[test]
    fn test_handle_key_event_release_ignored() {
        let event = handle_key_event(make_key_event(KeyCode::Char('q'), KeyEventKind::Release));
        assert_eq!(event, Event::None);
    }

    #[test]
    fn test_handle_key_event_unknown() {
        let event = handle_key_event(make_key_event(KeyCode::Char('x'), KeyEventKind::Press));
        assert_eq!(event, Event::None);
    }
}
