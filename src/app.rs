use crate::model::ApiSpec;

pub struct App {
    pub spec: ApiSpec,
    pub selected_index: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new(spec: ApiSpec) -> Self {
        Self {
            spec,
            selected_index: 0,
            should_quit: false,
        }
    }

    pub fn select_next(&mut self) {
        if !self.spec.endpoints.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.spec.endpoints.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.spec.endpoints.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.spec.endpoints.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
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
}
