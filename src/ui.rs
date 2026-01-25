use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Focus};
use crate::model::{Endpoint, HttpMethod, ParameterLocation};

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

fn status_code_color(status: &str) -> Color {
    match status.chars().next() {
        Some('2') => Color::Green,
        Some('3') => Color::Yellow,
        Some('4') => Color::Red,
        Some('5') => Color::Magenta,
        _ => Color::Gray,
    }
}

fn border_style(is_focused: bool) -> Style {
    if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(frame.area());

    // Left pane: Endpoint list
    render_endpoint_list(frame, app, chunks[0]);

    // Right pane: Detail view
    render_detail_view(frame, app, chunks[1]);
}

fn render_endpoint_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (list_area, search_area) = if app.search_mode {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .filter_map(|&idx| app.spec.endpoints.get(idx))
        .map(|endpoint| {
            let method_str = format!("{:width$}", endpoint.method, width = method_width());
            let line = Line::from(vec![
                Span::styled(
                    method_str,
                    Style::default().fg(method_color(&endpoint.method)),
                ),
                Span::raw(" "),
                Span::raw(&endpoint.path),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = if !app.search_query.is_empty() && !app.search_mode {
        format!(
            "{} v{} [{}] ({}/{})",
            app.spec.title,
            app.spec.version,
            app.search_query,
            app.filtered_indices.len(),
            app.spec.endpoints.len()
        )
    } else {
        format!("{} v{}", app.spec.title, app.spec.version)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style(app.focus == Focus::List && !app.search_mode)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, list_area, &mut list_state);

    if let Some(search_area) = search_area {
        render_search_bar(frame, app, search_area);
    }
}

fn render_search_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let search_text = format!("/{}", app.search_query);

    let paragraph = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(paragraph, area);

    let cursor_x = area.x + 2 + app.search_query.len() as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn render_detail_view(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let endpoint = app.selected_endpoint();

    let content = match endpoint {
        Some(ep) => build_detail_content(ep),
        None => Text::raw("No endpoint selected"),
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .border_style(border_style(app.focus == Focus::Detail)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn build_detail_content(endpoint: &Endpoint) -> Text<'static> {
    let mut lines: Vec<Line> = Vec::new();

    // Method + Path
    lines.push(Line::from(vec![
        Span::styled(
            endpoint.method.to_string(),
            Style::default()
                .fg(method_color(&endpoint.method))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            endpoint.path.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::raw(""));

    // Summary
    if let Some(summary) = &endpoint.summary {
        lines.push(Line::styled(
            summary.clone(),
            Style::default().fg(Color::White),
        ));
        lines.push(Line::raw(""));
    }

    // Description
    if let Some(description) = &endpoint.description {
        lines.push(Line::styled(
            description.clone(),
            Style::default().fg(Color::Gray),
        ));
        lines.push(Line::raw(""));
    }

    // Parameters
    if !endpoint.parameters.is_empty() {
        lines.push(Line::styled(
            "Parameters",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        // Group by location
        for location in &[
            ParameterLocation::Path,
            ParameterLocation::Query,
            ParameterLocation::Header,
            ParameterLocation::Cookie,
        ] {
            let params: Vec<_> = endpoint
                .parameters
                .iter()
                .filter(|p| &p.location == location)
                .collect();

            if !params.is_empty() {
                lines.push(Line::styled(
                    format!("  {}", location),
                    Style::default().fg(Color::DarkGray),
                ));

                for param in params {
                    let required_marker = if param.required { "*" } else { "" };
                    let type_str = param.schema_type.as_deref().unwrap_or("any");
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            format!("{}{}", param.name, required_marker),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(
                            format!(" ({})", type_str),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                    if let Some(desc) = &param.description {
                        lines.push(Line::styled(
                            format!("      {}", desc),
                            Style::default().fg(Color::Gray),
                        ));
                    }
                }
            }
        }
        lines.push(Line::raw(""));
    }

    // Request Body
    if let Some(body) = &endpoint.request_body {
        lines.push(Line::styled(
            format!(
                "Request Body{}",
                if body.required { " (required)" } else { "" }
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        if !body.content_types.is_empty() {
            lines.push(Line::styled(
                format!("  Content-Type: {}", body.content_types.join(", ")),
                Style::default().fg(Color::DarkGray),
            ));
        }

        if let Some(desc) = &body.description {
            lines.push(Line::styled(
                format!("  {}", desc),
                Style::default().fg(Color::Gray),
            ));
        }

        if let Some(schema) = &body.schema {
            lines.push(Line::styled(
                format!("  Schema: {}", schema),
                Style::default().fg(Color::Gray),
            ));
        }
        lines.push(Line::raw(""));
    }

    // Responses
    if !endpoint.responses.is_empty() {
        lines.push(Line::styled(
            "Responses",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        for (status, response) in &endpoint.responses {
            let status_color = status_code_color(status);

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(status.clone(), Style::default().fg(status_color)),
                Span::raw(" - "),
                Span::styled(
                    response.description.clone(),
                    Style::default().fg(Color::White),
                ),
            ]));

            if !response.content_types.is_empty() {
                lines.push(Line::styled(
                    format!("    Content-Type: {}", response.content_types.join(", ")),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if let Some(schema) = &response.schema {
                lines.push(Line::styled(
                    format!("    Schema: {}", schema),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
    }

    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Parameter, RequestBody, Response};
    use std::collections::BTreeMap;

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

    #[test]
    fn test_build_detail_content_basic() {
        let endpoint = Endpoint {
            method: HttpMethod::Get,
            path: "/users".to_string(),
            summary: Some("Get all users".to_string()),
            description: None,
            operation_id: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: BTreeMap::new(),
        };

        let content = build_detail_content(&endpoint);
        let text = content.to_string();

        assert!(text.contains("GET"));
        assert!(text.contains("/users"));
        assert!(text.contains("Get all users"));
    }

    #[test]
    fn test_build_detail_content_with_parameters() {
        let endpoint = Endpoint {
            method: HttpMethod::Get,
            path: "/users/{id}".to_string(),
            summary: None,
            description: None,
            operation_id: None,
            tags: vec![],
            parameters: vec![
                Parameter {
                    name: "id".to_string(),
                    location: ParameterLocation::Path,
                    description: Some("User ID".to_string()),
                    required: true,
                    schema_type: Some("integer".to_string()),
                },
                Parameter {
                    name: "include".to_string(),
                    location: ParameterLocation::Query,
                    description: None,
                    required: false,
                    schema_type: Some("string".to_string()),
                },
            ],
            request_body: None,
            responses: BTreeMap::new(),
        };

        let content = build_detail_content(&endpoint);
        let text = content.to_string();

        assert!(text.contains("Parameters"));
        assert!(text.contains("id*"));
        assert!(text.contains("(integer)"));
        assert!(text.contains("include"));
    }

    #[test]
    fn test_build_detail_content_with_request_body() {
        let endpoint = Endpoint {
            method: HttpMethod::Post,
            path: "/users".to_string(),
            summary: None,
            description: None,
            operation_id: None,
            tags: vec![],
            parameters: vec![],
            request_body: Some(RequestBody {
                description: Some("User data".to_string()),
                required: true,
                content_types: vec!["application/json".to_string()],
                schema: Some("User".to_string()),
            }),
            responses: BTreeMap::new(),
        };

        let content = build_detail_content(&endpoint);
        let text = content.to_string();

        assert!(text.contains("Request Body (required)"));
        assert!(text.contains("application/json"));
        assert!(text.contains("User data"));
    }

    #[test]
    fn test_build_detail_content_with_responses() {
        let mut responses = BTreeMap::new();
        responses.insert(
            "200".to_string(),
            Response {
                description: "Successful response".to_string(),
                content_types: vec!["application/json".to_string()],
                schema: Some("UserList".to_string()),
            },
        );
        responses.insert(
            "404".to_string(),
            Response {
                description: "Not found".to_string(),
                content_types: vec![],
                schema: None,
            },
        );

        let endpoint = Endpoint {
            method: HttpMethod::Get,
            path: "/users".to_string(),
            summary: None,
            description: None,
            operation_id: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses,
        };

        let content = build_detail_content(&endpoint);
        let text = content.to_string();

        assert!(text.contains("Responses"));
        assert!(text.contains("200"));
        assert!(text.contains("Successful response"));
        assert!(text.contains("404"));
        assert!(text.contains("Not found"));
    }
}
