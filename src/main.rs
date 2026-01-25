mod app;
mod event;
mod model;
mod parser;
mod ui;

use std::env;
use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, Focus};
use event::Event;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: oatui <openapi-file>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let spec = parser::parse_file(file_path)?;
    let mut app = App::new(spec);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        let event = event::poll_event(Duration::from_millis(100))?;
        handle_event(app, event);

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_event(app: &mut App, event: Event) {
    if app.search_mode {
        handle_search_mode_event(app, event);
        return;
    }

    match event {
        Event::Quit => app.quit(),
        Event::Search => app.enter_search_mode(),
        Event::Enter => app.focus_detail(),
        Event::Back if app.focus == Focus::Detail => app.focus_list(),
        Event::Back if !app.search_query.is_empty() => app.clear_search(),
        Event::NavigateDown if app.focus == Focus::List => app.select_next(),
        Event::NavigateDown => app.scroll_down(),
        Event::NavigateUp if app.focus == Focus::List => app.select_previous(),
        Event::NavigateUp => app.scroll_up(),
        Event::Back | Event::None | Event::Char(_) | Event::Backspace => {}
    }
}

fn handle_search_mode_event(app: &mut App, event: Event) {
    match event {
        Event::Back => app.cancel_search(),
        Event::Enter => app.confirm_search(),
        Event::Char(c) => app.search_push_char(c),
        Event::Backspace => app.search_pop_char(),
        Event::NavigateDown => app.select_next(),
        Event::NavigateUp => app.select_previous(),
        Event::Quit | Event::Search | Event::None => {}
    }
}
