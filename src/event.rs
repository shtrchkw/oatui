use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Quit,
    NavigateUp,
    NavigateDown,
    Enter,
    Back,
    Search,
    Char(char),
    Backspace,
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
        KeyCode::Char('/') => Event::Search,
        KeyCode::Esc => Event::Back,
        KeyCode::Enter => Event::Enter,
        KeyCode::Backspace => Event::Backspace,
        KeyCode::Down | KeyCode::Char('j') => Event::NavigateDown,
        KeyCode::Up | KeyCode::Char('k') => Event::NavigateUp,
        KeyCode::Char(c) => Event::Char(c),
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
    fn test_handle_key_event_char() {
        let event = handle_key_event(make_key_event(KeyCode::Char('x'), KeyEventKind::Press));
        assert_eq!(event, Event::Char('x'));

        let event = handle_key_event(make_key_event(KeyCode::Char('a'), KeyEventKind::Press));
        assert_eq!(event, Event::Char('a'));
    }

    #[test]
    fn test_handle_key_event_search() {
        let event = handle_key_event(make_key_event(KeyCode::Char('/'), KeyEventKind::Press));
        assert_eq!(event, Event::Search);
    }

    #[test]
    fn test_handle_key_event_backspace() {
        let event = handle_key_event(make_key_event(KeyCode::Backspace, KeyEventKind::Press));
        assert_eq!(event, Event::Backspace);
    }

    #[test]
    fn test_handle_key_event_unknown() {
        let event = handle_key_event(make_key_event(KeyCode::Tab, KeyEventKind::Press));
        assert_eq!(event, Event::None);
    }
}
