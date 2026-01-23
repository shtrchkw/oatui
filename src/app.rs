use crate::model::ApiSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

pub struct App {
    pub spec: ApiSpec,
    pub selected_index: usize,
    pub should_quit: bool,
    pub focus: Focus,
    pub detail_scroll: u16,
}

impl App {
    pub fn new(spec: ApiSpec) -> Self {
        Self {
            spec,
            selected_index: 0,
            should_quit: false,
            focus: Focus::List,
            detail_scroll: 0,
        }
    }

    pub fn select_next(&mut self) {
        let len = self.spec.endpoints.len();
        if len > 0 {
            self.selected_index = (self.selected_index + 1) % len;
            self.detail_scroll = 0;
        }
    }

    pub fn select_previous(&mut self) {
        let len = self.spec.endpoints.len();
        if len > 0 {
            self.selected_index = self.selected_index.checked_sub(1).unwrap_or(len - 1);
            self.detail_scroll = 0;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn focus_detail(&mut self) {
        self.focus = Focus::Detail;
    }

    pub fn focus_list(&mut self) {
        self.focus = Focus::List;
    }

    pub fn scroll_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Endpoint, HttpMethod};
    use std::collections::BTreeMap;

    fn create_test_spec(endpoint_count: usize) -> ApiSpec {
        let endpoints: Vec<Endpoint> = (0..endpoint_count)
            .map(|i| Endpoint {
                method: HttpMethod::Get,
                path: format!("/test/{}", i),
                summary: None,
                description: None,
                operation_id: None,
                tags: vec![],
                parameters: vec![],
                request_body: None,
                responses: BTreeMap::new(),
            })
            .collect();

        ApiSpec {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            endpoints,
        }
    }

    #[test]
    fn test_new_app() {
        let spec = create_test_spec(3);
        let app = App::new(spec);
        assert_eq!(app.selected_index, 0);
        assert!(!app.should_quit);
        assert_eq!(app.focus, Focus::List);
        assert_eq!(app.detail_scroll, 0);
    }

    #[test]
    fn test_select_next() {
        let spec = create_test_spec(3);
        let mut app = App::new(spec);

        app.select_next();
        assert_eq!(app.selected_index, 1);

        app.select_next();
        assert_eq!(app.selected_index, 2);

        app.select_next();
        assert_eq!(app.selected_index, 0); // wraps around
    }

    #[test]
    fn test_select_previous() {
        let spec = create_test_spec(3);
        let mut app = App::new(spec);

        app.select_previous();
        assert_eq!(app.selected_index, 2); // wraps to end

        app.select_previous();
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_select_with_empty_endpoints() {
        let spec = create_test_spec(0);
        let mut app = App::new(spec);

        app.select_next();
        assert_eq!(app.selected_index, 0);

        app.select_previous();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_quit() {
        let spec = create_test_spec(1);
        let mut app = App::new(spec);

        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_focus_detail() {
        let spec = create_test_spec(1);
        let mut app = App::new(spec);

        assert_eq!(app.focus, Focus::List);
        app.focus_detail();
        assert_eq!(app.focus, Focus::Detail);
    }

    #[test]
    fn test_focus_list() {
        let spec = create_test_spec(1);
        let mut app = App::new(spec);

        app.focus_detail();
        assert_eq!(app.focus, Focus::Detail);
        app.focus_list();
        assert_eq!(app.focus, Focus::List);
    }

    #[test]
    fn test_scroll_down() {
        let spec = create_test_spec(1);
        let mut app = App::new(spec);

        assert_eq!(app.detail_scroll, 0);
        app.scroll_down();
        assert_eq!(app.detail_scroll, 1);
        app.scroll_down();
        assert_eq!(app.detail_scroll, 2);
    }

    #[test]
    fn test_scroll_up() {
        let spec = create_test_spec(1);
        let mut app = App::new(spec);

        app.detail_scroll = 5;
        app.scroll_up();
        assert_eq!(app.detail_scroll, 4);
        app.scroll_up();
        assert_eq!(app.detail_scroll, 3);
    }

    #[test]
    fn test_scroll_up_at_zero() {
        let spec = create_test_spec(1);
        let mut app = App::new(spec);

        assert_eq!(app.detail_scroll, 0);
        app.scroll_up();
        assert_eq!(app.detail_scroll, 0); // should not underflow
    }

    #[test]
    fn test_select_resets_scroll() {
        let spec = create_test_spec(3);
        let mut app = App::new(spec);

        app.detail_scroll = 10;
        app.select_next();
        assert_eq!(app.detail_scroll, 0);

        app.detail_scroll = 10;
        app.select_previous();
        assert_eq!(app.detail_scroll, 0);
    }
}
