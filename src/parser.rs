use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use openapiv3::{
    OpenAPI, Operation, Parameter as OApiParameter, ParameterSchemaOrContent, PathItem,
    ReferenceOr, Schema, StatusCode, Type,
};

use crate::model::{
    ApiSpec, Endpoint, HttpMethod, Parameter, ParameterLocation, RequestBody, Response,
};

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ApiSpec> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let openapi: OpenAPI = match extension.to_lowercase().as_str() {
        "json" => serde_json::from_str(&content).with_context(|| "Failed to parse JSON")?,
        "yaml" | "yml" => serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML")?,
        _ => {
            // Try YAML first, then JSON
            serde_yaml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content))
                .with_context(|| "Failed to parse file as YAML or JSON")?
        }
    };

    convert_openapi(openapi)
}

fn convert_openapi(openapi: OpenAPI) -> Result<ApiSpec> {
    let mut endpoints = Vec::new();

    for (path, path_item) in &openapi.paths.paths {
        if let ReferenceOr::Item(item) = path_item {
            endpoints.extend(extract_endpoints(path, item, &openapi));
        }
    }

    // Sort endpoints by path, then by method
    endpoints.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| method_order(&a.method).cmp(&method_order(&b.method)))
    });

    Ok(ApiSpec {
        title: openapi.info.title.clone(),
        version: openapi.info.version.clone(),
        description: openapi.info.description.clone(),
        endpoints,
    })
}

fn method_order(method: &HttpMethod) -> u8 {
    match method {
        HttpMethod::Get => 0,
        HttpMethod::Post => 1,
        HttpMethod::Put => 2,
        HttpMethod::Patch => 3,
        HttpMethod::Delete => 4,
        HttpMethod::Head => 5,
        HttpMethod::Options => 6,
        HttpMethod::Trace => 7,
    }
}

fn extract_endpoints(path: &str, item: &PathItem, openapi: &OpenAPI) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();

    let operations = [
        (HttpMethod::Get, &item.get),
        (HttpMethod::Post, &item.post),
        (HttpMethod::Put, &item.put),
        (HttpMethod::Delete, &item.delete),
        (HttpMethod::Patch, &item.patch),
        (HttpMethod::Head, &item.head),
        (HttpMethod::Options, &item.options),
        (HttpMethod::Trace, &item.trace),
    ];

    for (method, op) in operations {
        if let Some(operation) = op {
            endpoints.push(convert_operation(
                path,
                method,
                operation,
                &item.parameters,
                openapi,
            ));
        }
    }

    endpoints
}

fn convert_operation(
    path: &str,
    method: HttpMethod,
    op: &Operation,
    path_params: &[ReferenceOr<OApiParameter>],
    openapi: &OpenAPI,
) -> Endpoint {
    // Use HashMap to handle parameter override (operation-level overrides path-level)
    // Key: (name, location)
    let mut param_map: HashMap<(String, ParameterLocation), Parameter> = HashMap::new();

    // Add path-level parameters first
    for param in path_params {
        if let Some(p) = convert_parameter(param, openapi) {
            param_map.insert((p.name.clone(), p.location.clone()), p);
        }
    }

    // Add operation-level parameters (overrides path-level if same name+location)
    for param in &op.parameters {
        if let Some(p) = convert_parameter(param, openapi) {
            param_map.insert((p.name.clone(), p.location.clone()), p);
        }
    }

    let parameters: Vec<Parameter> = param_map.into_values().collect();

    let request_body = op.request_body.as_ref().and_then(|rb| {
        let body = resolve_request_body(rb, openapi)?;
        let content_types: Vec<String> = body.content.keys().cloned().collect();
        let schema = body
            .content
            .values()
            .next()
            .and_then(|mt| mt.schema.as_ref())
            .and_then(|s| schema_type_to_string(s, openapi));

        Some(RequestBody {
            description: body.description.clone(),
            required: body.required,
            content_types,
            schema,
        })
    });

    let mut responses = BTreeMap::new();
    for (status, response) in &op.responses.responses {
        let status_code = match status {
            StatusCode::Code(code) => code.to_string(),
            StatusCode::Range(range) => format!("{}XX", range),
        };

        if let Some(resp) = resolve_response(response, openapi) {
            responses.insert(status_code.clone(), convert_response(&status_code, resp, openapi));
        }
    }

    if let Some(resp) = op
        .responses
        .default
        .as_ref()
        .and_then(|r| resolve_response(r, openapi))
    {
        responses.insert("default".to_string(), convert_response("default", resp, openapi));
    }

    Endpoint {
        method,
        path: path.to_string(),
        summary: op.summary.clone(),
        description: op.description.clone(),
        operation_id: op.operation_id.clone(),
        tags: op.tags.clone(),
        parameters,
        request_body,
        responses,
    }
}

fn convert_parameter(param: &ReferenceOr<OApiParameter>, openapi: &OpenAPI) -> Option<Parameter> {
    let param = resolve_parameter(param, openapi)?;

    let (location, parameter_data) = match param {
        OApiParameter::Path { parameter_data, .. } => (ParameterLocation::Path, parameter_data),
        OApiParameter::Query { parameter_data, .. } => (ParameterLocation::Query, parameter_data),
        OApiParameter::Header { parameter_data, .. } => (ParameterLocation::Header, parameter_data),
        OApiParameter::Cookie { parameter_data, .. } => (ParameterLocation::Cookie, parameter_data),
    };

    let schema_type = match &parameter_data.format {
        ParameterSchemaOrContent::Schema(s) => schema_type_to_string(s, openapi),
        ParameterSchemaOrContent::Content(_) => None,
    };

    Some(Parameter {
        name: parameter_data.name.clone(),
        location,
        description: parameter_data.description.clone(),
        required: parameter_data.required,
        schema_type,
    })
}

/// Resolves a `ReferenceOr<T>` to `&T` by looking up the component if it's a reference.
fn resolve_ref<'a, T, F>(
    ref_or_item: &'a ReferenceOr<T>,
    prefix: &str,
    get_component: F,
) -> Option<&'a T>
where
    F: FnOnce(&str) -> Option<&'a ReferenceOr<T>>,
{
    match ref_or_item {
        ReferenceOr::Item(item) => Some(item),
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix(prefix)?;
            match get_component(name)? {
                ReferenceOr::Item(item) => Some(item),
                ReferenceOr::Reference { .. } => None,
            }
        }
    }
}

fn resolve_parameter<'a>(
    param: &'a ReferenceOr<OApiParameter>,
    openapi: &'a OpenAPI,
) -> Option<&'a OApiParameter> {
    resolve_ref(param, "#/components/parameters/", |name| {
        openapi.components.as_ref()?.parameters.get(name)
    })
}

fn resolve_request_body<'a>(
    rb: &'a ReferenceOr<openapiv3::RequestBody>,
    openapi: &'a OpenAPI,
) -> Option<&'a openapiv3::RequestBody> {
    resolve_ref(rb, "#/components/requestBodies/", |name| {
        openapi.components.as_ref()?.request_bodies.get(name)
    })
}

fn resolve_response<'a>(
    resp: &'a ReferenceOr<openapiv3::Response>,
    openapi: &'a OpenAPI,
) -> Option<&'a openapiv3::Response> {
    resolve_ref(resp, "#/components/responses/", |name| {
        openapi.components.as_ref()?.responses.get(name)
    })
}

fn convert_response(_status_code: &str, resp: &openapiv3::Response, openapi: &OpenAPI) -> Response {
    let content_types: Vec<String> = resp.content.keys().cloned().collect();
    let schema = resp
        .content
        .values()
        .next()
        .and_then(|mt| mt.schema.as_ref())
        .and_then(|s| schema_type_to_string(s, openapi));

    Response {
        description: resp.description.clone(),
        content_types,
        schema,
    }
}

fn schema_type_to_string(schema: &ReferenceOr<Schema>, _openapi: &OpenAPI) -> Option<String> {
    match schema {
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/schemas/")?;
            Some(name.to_string())
        }
        ReferenceOr::Item(schema) => match &schema.schema_kind {
            openapiv3::SchemaKind::Type(t) => Some(type_to_string(t).to_string()),
            openapiv3::SchemaKind::Any(any) => any.typ.clone(),
            _ => None,
        },
    }
}

fn type_to_string(t: &Type) -> &'static str {
    match t {
        Type::String(_) => "string",
        Type::Number(_) => "number",
        Type::Integer(_) => "integer",
        Type::Boolean(_) => "boolean",
        Type::Array(_) => "array",
        Type::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_file() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        assert_eq!(spec.title, "Petstore API");
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(
            spec.description,
            Some("A sample API for testing oatui".to_string())
        );
    }

    #[test]
    fn test_endpoint_count() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        assert_eq!(spec.endpoints.len(), 5);
    }

    #[test]
    fn test_endpoint_methods() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let methods: Vec<_> = spec.endpoints.iter().map(|e| e.method.clone()).collect();

        assert!(methods.contains(&HttpMethod::Get));
        assert!(methods.contains(&HttpMethod::Post));
        assert!(methods.contains(&HttpMethod::Put));
        assert!(methods.contains(&HttpMethod::Delete));
    }

    #[test]
    fn test_endpoint_paths() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let paths: Vec<_> = spec.endpoints.iter().map(|e| e.path.as_str()).collect();

        assert!(paths.contains(&"/pets"));
        assert!(paths.contains(&"/pets/{petId}"));
    }

    #[test]
    fn test_endpoint_sorting() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        // Endpoints should be sorted by path, then by method order
        assert_eq!(spec.endpoints[0].path, "/pets");
        assert_eq!(spec.endpoints[0].method, HttpMethod::Get);
        assert_eq!(spec.endpoints[1].path, "/pets");
        assert_eq!(spec.endpoints[1].method, HttpMethod::Post);
    }

    #[test]
    fn test_endpoint_summary() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let list_pets = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/pets" && e.method == HttpMethod::Get)
            .unwrap();

        assert_eq!(list_pets.summary, Some("List all pets".to_string()));
    }

    #[test]
    fn test_endpoint_parameters() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let list_pets = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/pets" && e.method == HttpMethod::Get)
            .unwrap();

        assert_eq!(list_pets.parameters.len(), 1);
        assert_eq!(list_pets.parameters[0].name, "limit");
        assert_eq!(list_pets.parameters[0].location, ParameterLocation::Query);
    }

    #[test]
    fn test_path_parameters() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let get_pet = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/pets/{petId}" && e.method == HttpMethod::Get)
            .unwrap();

        let pet_id_param = get_pet
            .parameters
            .iter()
            .find(|p| p.name == "petId")
            .unwrap();

        assert_eq!(pet_id_param.location, ParameterLocation::Path);
        assert!(pet_id_param.required);
    }

    #[test]
    fn test_request_body() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let create_pet = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/pets" && e.method == HttpMethod::Post)
            .unwrap();

        assert!(create_pet.request_body.is_some());
        let body = create_pet.request_body.as_ref().unwrap();
        assert!(body.required);
        assert!(body.content_types.contains(&"application/json".to_string()));
    }

    #[test]
    fn test_responses() {
        let spec = parse_file("tests/fixtures/petstore.yaml").unwrap();

        let get_pet = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/pets/{petId}" && e.method == HttpMethod::Get)
            .unwrap();

        assert!(get_pet.responses.contains_key("200"));
        assert!(get_pet.responses.contains_key("404"));
    }

    #[test]
    fn test_nonexistent_file() {
        let result = parse_file("nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
        assert_eq!(format!("{}", HttpMethod::Post), "POST");
        assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
    }

    #[test]
    fn test_parameter_location_display() {
        assert_eq!(format!("{}", ParameterLocation::Path), "path");
        assert_eq!(format!("{}", ParameterLocation::Query), "query");
        assert_eq!(format!("{}", ParameterLocation::Header), "header");
    }

    #[test]
    fn test_parameter_override_uses_path_level_by_default() {
        let spec = parse_file("tests/fixtures/parameter-override.yaml").unwrap();

        // GET uses path-level parameters (no override)
        let get_item = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/items/{itemId}" && e.method == HttpMethod::Get)
            .unwrap();

        assert_eq!(get_item.parameters.len(), 2);

        let item_id = get_item
            .parameters
            .iter()
            .find(|p| p.name == "itemId")
            .unwrap();
        assert_eq!(item_id.description, Some("Path level item ID".to_string()));
        assert_eq!(item_id.schema_type, Some("string".to_string()));
    }

    #[test]
    fn test_parameter_override_operation_overrides_path() {
        let spec = parse_file("tests/fixtures/parameter-override.yaml").unwrap();

        // PUT overrides itemId parameter
        let put_item = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/items/{itemId}" && e.method == HttpMethod::Put)
            .unwrap();

        // Should have 2 params: overridden itemId + inherited version
        assert_eq!(put_item.parameters.len(), 2);

        let item_id = put_item
            .parameters
            .iter()
            .find(|p| p.name == "itemId")
            .unwrap();
        assert_eq!(
            item_id.description,
            Some("Operation level item ID (overridden)".to_string())
        );
        assert_eq!(item_id.schema_type, Some("integer".to_string()));
    }

    #[test]
    fn test_parameter_override_can_change_required() {
        let spec = parse_file("tests/fixtures/parameter-override.yaml").unwrap();

        // DELETE overrides version parameter (required: false -> true)
        let delete_item = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/items/{itemId}" && e.method == HttpMethod::Delete)
            .unwrap();

        let version = delete_item
            .parameters
            .iter()
            .find(|p| p.name == "version")
            .unwrap();
        assert!(version.required);
        assert_eq!(
            version.description,
            Some("Required version for delete".to_string())
        );
    }

    #[test]
    fn test_resolve_request_body_ref() {
        let spec = parse_file("tests/fixtures/ref-test.yaml").unwrap();

        let create_user = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/users" && e.method == HttpMethod::Post)
            .unwrap();

        assert!(create_user.request_body.is_some());
        let body = create_user.request_body.as_ref().unwrap();
        assert!(body.required);
        assert_eq!(body.description, Some("User data to create".to_string()));
        assert!(body.content_types.contains(&"application/json".to_string()));
    }

    #[test]
    fn test_resolve_response_ref() {
        let spec = parse_file("tests/fixtures/ref-test.yaml").unwrap();

        let create_user = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/users" && e.method == HttpMethod::Post)
            .unwrap();

        assert!(create_user.responses.contains_key("201"));
        let resp_201 = create_user.responses.get("201").unwrap();
        assert_eq!(resp_201.description, "User created successfully");

        assert!(create_user.responses.contains_key("400"));
        let resp_400 = create_user.responses.get("400").unwrap();
        assert_eq!(resp_400.description, "Invalid request");
    }

    #[test]
    fn test_resolve_default_response_ref() {
        let spec = parse_file("tests/fixtures/ref-test.yaml").unwrap();

        let get_user = spec
            .endpoints
            .iter()
            .find(|e| e.path == "/users/{id}" && e.method == HttpMethod::Get)
            .unwrap();

        assert!(get_user.responses.contains_key("default"));
        let default_resp = get_user.responses.get("default").unwrap();
        assert_eq!(default_resp.description, "Unexpected error");
    }
}
