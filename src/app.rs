use crate::model::{ApiSpec, Endpoint};

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
    pub search_mode: bool,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
}

impl App {
    pub fn new(spec: ApiSpec) -> Self {
        let endpoint_count = spec.endpoints.len();
        Self {
            spec,
            selected_index: 0,
            should_quit: false,
            focus: Focus::List,
            detail_scroll: 0,
            search_mode: false,
            search_query: String::new(),
            filtered_indices: (0..endpoint_count).collect(),
        }
    }

    pub fn select_next(&mut self) {
        let len = self.filtered_indices.len();
        if len > 0 {
            self.selected_index = (self.selected_index + 1) % len;
            self.detail_scroll = 0;
        }
    }

    pub fn select_previous(&mut self) {
        let len = self.filtered_indices.len();
        if len > 0 {
            self.selected_index = self.selected_index.checked_sub(1).unwrap_or(len - 1);
            self.detail_scroll = 0;
        }
    }

    pub fn selected_endpoint(&self) -> Option<&Endpoint> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.spec.endpoints.get(idx))
    }

    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.focus = Focus::List;
    }

    pub fn cancel_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.update_filtered_indices();
    }

    pub fn confirm_search(&mut self) {
        self.search_mode = false;
    }

    pub fn search_push_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_indices();
    }

    pub fn search_pop_char(&mut self) {
        self.search_query.pop();
        self.update_filtered_indices();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_filtered_indices();
        self.selected_index = 0;
    }

    fn update_filtered_indices(&mut self) {
        let query_lower = self.search_query.to_lowercase();

        self.filtered_indices = self
            .spec
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, ep)| {
                query_lower.is_empty() || ep.path.to_lowercase().contains(&query_lower)
            })
            .map(|(i, _)| i)
            .collect();

        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
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

    #[test]
    fn test_new_app_initializes_filtered_indices() {
        let spec = create_test_spec(3);
        let app = App::new(spec);
        assert_eq!(app.filtered_indices, vec![0, 1, 2]);
        assert!(!app.search_mode);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn test_search_mode_toggle() {
        let spec = create_test_spec(3);
        let mut app = App::new(spec);

        assert!(!app.search_mode);
        app.enter_search_mode();
        assert!(app.search_mode);
        app.confirm_search();
        assert!(!app.search_mode);
    }

    #[test]
    fn test_search_cancel_clears_query() {
        let spec = create_test_spec(3);
        let mut app = App::new(spec);

        app.enter_search_mode();
        app.search_push_char('t');
        app.search_push_char('e');
        app.cancel_search();

        assert!(!app.search_mode);
        assert!(app.search_query.is_empty());
        assert_eq!(app.filtered_indices.len(), 3);
    }

    fn create_endpoint_with_path(path: &str) -> Endpoint {
        Endpoint {
            method: HttpMethod::Get,
            path: path.to_string(),
            summary: None,
            description: None,
            operation_id: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: BTreeMap::new(),
        }
    }

    #[test]
    fn test_search_filters_endpoints() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            endpoints: vec![
                create_endpoint_with_path("/users"),
                create_endpoint_with_path("/users/{id}"),
                create_endpoint_with_path("/posts"),
            ],
        };
        let mut app = App::new(spec);

        app.search_push_char('u');
        app.search_push_char('s');
        app.search_push_char('e');
        app.search_push_char('r');

        assert_eq!(app.filtered_indices.len(), 2);
        assert_eq!(app.filtered_indices, vec![0, 1]);
    }

    #[test]
    fn test_search_case_insensitive() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            endpoints: vec![
                create_endpoint_with_path("/Users"),
                create_endpoint_with_path("/ADMIN"),
            ],
        };
        let mut app = App::new(spec);

        app.search_push_char('u');
        app.search_push_char('s');

        assert_eq!(app.filtered_indices.len(), 1);
        assert_eq!(app.filtered_indices, vec![0]);
    }

    #[test]
    fn test_search_resets_selection_when_out_of_bounds() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            endpoints: vec![
                create_endpoint_with_path("/a"),
                create_endpoint_with_path("/b"),
                create_endpoint_with_path("/c"),
            ],
        };
        let mut app = App::new(spec);

        app.selected_index = 2;
        app.search_push_char('a');

        assert_eq!(app.filtered_indices.len(), 1);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_selected_endpoint_returns_correct_endpoint() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            endpoints: vec![
                create_endpoint_with_path("/aaa"),
                create_endpoint_with_path("/bbb"),
                create_endpoint_with_path("/ccc"),
            ],
        };
        let mut app = App::new(spec);

        app.search_push_char('b');

        let selected = app.selected_endpoint().unwrap();
        assert_eq!(selected.path, "/bbb");
    }

    #[test]
    fn test_navigation_wraps_in_filtered_list() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            endpoints: vec![
                create_endpoint_with_path("/a"),
                create_endpoint_with_path("/b"),
                create_endpoint_with_path("/ab"),
            ],
        };
        let mut app = App::new(spec);

        app.search_push_char('a');

        assert_eq!(app.filtered_indices.len(), 2);
        assert_eq!(app.filtered_indices, vec![0, 2]);
        app.select_next();
        assert_eq!(app.selected_index, 1);
        app.select_next();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_backspace_in_search() {
        let spec = create_test_spec(3);
        let mut app = App::new(spec);

        app.search_push_char('a');
        app.search_push_char('b');
        assert_eq!(app.search_query, "ab");

        app.search_pop_char();
        assert_eq!(app.search_query, "a");
    }

    #[test]
    fn test_clear_search_shows_all() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            endpoints: vec![
                create_endpoint_with_path("/a"),
                create_endpoint_with_path("/b"),
            ],
        };
        let mut app = App::new(spec);

        app.search_push_char('a');
        assert_eq!(app.filtered_indices.len(), 1);

        app.clear_search();
        assert_eq!(app.filtered_indices.len(), 2);
        assert!(app.search_query.is_empty());
    }
}
