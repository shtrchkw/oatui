use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use openapiv3::{OpenAPI, Operation, Parameter as OApiParameter, ParameterSchemaOrContent, PathItem, ReferenceOr, Schema, StatusCode, Type};

use crate::model::{ApiSpec, Endpoint, HttpMethod, Parameter, ParameterLocation, RequestBody, Response};

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ApiSpec> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let openapi: OpenAPI = match extension.to_lowercase().as_str() {
        "json" => serde_json::from_str(&content)
            .with_context(|| "Failed to parse JSON")?,
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse YAML")?,
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
        a.path.cmp(&b.path).then_with(|| method_order(&a.method).cmp(&method_order(&b.method)))
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
            endpoints.push(convert_operation(path, method, operation, &item.parameters, openapi));
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
    let mut parameters = Vec::new();

    // Add path-level parameters
    for param in path_params {
        if let Some(p) = convert_parameter(param, openapi) {
            parameters.push(p);
        }
    }

    // Add operation-level parameters
    for param in &op.parameters {
        if let Some(p) = convert_parameter(param, openapi) {
            parameters.push(p);
        }
    }

    let request_body = op.request_body.as_ref().and_then(|rb| {
        match rb {
            ReferenceOr::Item(body) => {
                let content_types: Vec<String> = body.content.keys().cloned().collect();
                let schema = body.content.values().next()
                    .and_then(|mt| mt.schema.as_ref())
                    .and_then(|s| schema_to_string(s, openapi));

                Some(RequestBody {
                    description: body.description.clone(),
                    required: body.required,
                    content_types,
                    schema,
                })
            }
            ReferenceOr::Reference { .. } => None,
        }
    });

    let mut responses = BTreeMap::new();
    for (status, response) in &op.responses.responses {
        let status_code = match status {
            StatusCode::Code(code) => code.to_string(),
            StatusCode::Range(range) => format!("{}XX", range),
        };

        if let ReferenceOr::Item(resp) = response {
            let content_types: Vec<String> = resp.content.keys().cloned().collect();
            let schema = resp.content.values().next()
                .and_then(|mt| mt.schema.as_ref())
                .and_then(|s| schema_to_string(s, openapi));

            responses.insert(status_code.clone(), Response {
                status_code,
                description: resp.description.clone(),
                content_types,
                schema,
            });
        }
    }

    // Handle default response
    if let Some(ReferenceOr::Item(resp)) = &op.responses.default {
        let content_types: Vec<String> = resp.content.keys().cloned().collect();
        let schema = resp.content.values().next()
            .and_then(|mt| mt.schema.as_ref())
            .and_then(|s| schema_to_string(s, openapi));

        responses.insert("default".to_string(), Response {
            status_code: "default".to_string(),
            description: resp.description.clone(),
            content_types,
            schema,
        });
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

    let location = match param {
        OApiParameter::Path { .. } => ParameterLocation::Path,
        OApiParameter::Query { .. } => ParameterLocation::Query,
        OApiParameter::Header { .. } => ParameterLocation::Header,
        OApiParameter::Cookie { .. } => ParameterLocation::Cookie,
    };

    let (name, description, required, schema) = match param {
        OApiParameter::Path { parameter_data, .. }
        | OApiParameter::Query { parameter_data, .. }
        | OApiParameter::Header { parameter_data, .. }
        | OApiParameter::Cookie { parameter_data, .. } => {
            let schema_type = match &parameter_data.format {
                ParameterSchemaOrContent::Schema(s) => schema_type_to_string(s, openapi),
                ParameterSchemaOrContent::Content(_) => None,
            };
            (
                parameter_data.name.clone(),
                parameter_data.description.clone(),
                parameter_data.required,
                schema_type,
            )
        }
    };

    Some(Parameter {
        name,
        location,
        description,
        required,
        schema_type: schema,
    })
}

fn resolve_parameter<'a>(param: &'a ReferenceOr<OApiParameter>, openapi: &'a OpenAPI) -> Option<&'a OApiParameter> {
    match param {
        ReferenceOr::Item(p) => Some(p),
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/parameters/")?;
            openapi.components.as_ref()?.parameters.get(name).and_then(|p| {
                match p {
                    ReferenceOr::Item(p) => Some(p),
                    _ => None,
                }
            })
        }
    }
}

fn schema_type_to_string(schema: &ReferenceOr<Schema>, _openapi: &OpenAPI) -> Option<String> {
    match schema {
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/schemas/")?;
            Some(name.to_string())
        }
        ReferenceOr::Item(schema) => {
            match &schema.schema_kind {
                openapiv3::SchemaKind::Type(t) => Some(type_to_string(t)),
                openapiv3::SchemaKind::Any(any) => {
                    any.typ.clone()
                }
                _ => None,
            }
        }
    }
}

fn schema_to_string(schema: &ReferenceOr<Schema>, openapi: &OpenAPI) -> Option<String> {
    schema_type_to_string(schema, openapi)
}

fn type_to_string(t: &Type) -> String {
    match t {
        Type::String(_) => "string".to_string(),
        Type::Number(_) => "number".to_string(),
        Type::Integer(_) => "integer".to_string(),
        Type::Boolean(_) => "boolean".to_string(),
        Type::Array(_) => "array".to_string(),
        Type::Object(_) => "object".to_string(),
    }
}
