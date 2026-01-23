use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => f.write_str("GET"),
            HttpMethod::Post => f.write_str("POST"),
            HttpMethod::Put => f.write_str("PUT"),
            HttpMethod::Delete => f.write_str("DELETE"),
            HttpMethod::Patch => f.write_str("PATCH"),
            HttpMethod::Head => f.write_str("HEAD"),
            HttpMethod::Options => f.write_str("OPTIONS"),
            HttpMethod::Trace => f.write_str("TRACE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

impl std::fmt::Display for ParameterLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterLocation::Path => f.write_str("path"),
            ParameterLocation::Query => f.write_str("query"),
            ParameterLocation::Header => f.write_str("header"),
            ParameterLocation::Cookie => f.write_str("cookie"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
    pub description: Option<String>,
    pub required: bool,
    pub schema_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: bool,
    pub content_types: Vec<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: String,
    pub description: String,
    pub content_types: Vec<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: BTreeMap<String, Response>,
}

#[derive(Debug, Clone)]
pub struct ApiSpec {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub endpoints: Vec<Endpoint>,
}
