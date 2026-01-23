use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::model::HttpMethod;

fn method_color(method: &HttpMethod) -> Color {
    match method {
        HttpMethod::Get => Color::Green,
        HttpMethod::Post => Color::Blue,
        HttpMethod::Put => Color::Yellow,
        HttpMethod::Delete => Color::Red,
        HttpMethod::Patch => Color::Cyan,
        HttpMethod::Head => Color::Magenta,
        HttpMethod::Options => Color::Gray,
        HttpMethod::Trace => Color::Gray,
    }
}

fn method_width() -> usize {
    7 // "OPTIONS" is the longest method name
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(frame.area());

    // Left pane: Endpoint list
    let items: Vec<ListItem> = app
        .spec
        .endpoints
        .iter()
        .map(|endpoint| {
            let method_str = format!("{:width$}", endpoint.method, width = method_width());
            let line = Line::from(vec![
                Span::styled(method_str, Style::default().fg(method_color(&endpoint.method))),
                Span::raw(" "),
                Span::raw(&endpoint.path),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!("{} v{}", app.spec.title, app.spec.version);
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    // Right pane: Placeholder for detail view
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title("Details");
    frame.render_widget(detail_block, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_color() {
        assert_eq!(method_color(&HttpMethod::Get), Color::Green);
        assert_eq!(method_color(&HttpMethod::Post), Color::Blue);
        assert_eq!(method_color(&HttpMethod::Put), Color::Yellow);
        assert_eq!(method_color(&HttpMethod::Delete), Color::Red);
        assert_eq!(method_color(&HttpMethod::Patch), Color::Cyan);
    }

    #[test]
    fn test_method_width() {
        // Ensure width accommodates all method names
        assert!(method_width() >= "GET".len());
        assert!(method_width() >= "POST".len());
        assert!(method_width() >= "DELETE".len());
        assert!(method_width() >= "OPTIONS".len());
    }
}
