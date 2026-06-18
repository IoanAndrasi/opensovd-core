// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

//! SOVD online capability descriptions.
//!
//! `GET /{path}/docs` returns a self-contained OpenAPI 3.1 document for a single
//! endpoint: its methods, payload schemas and status codes. A client can use it
//! without an offline capability description for the whole vehicle.
//!
//! Data collection endpoints are wired up so far; see [`data_collection_docs`].

use opensovd_models::data::{DataList, Metadata};
use serde_json::{Value, json};

use crate::schema::JsonSchema;

/// Wrap a single path item into a self-contained OpenAPI 3.1 document.
pub fn build_openapi_doc(title: &str, path: &str, path_item: Value) -> Value {
    let mut paths = serde_json::Map::new();
    paths.insert(path.to_owned(), path_item);

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": title,
            "version": "1.0.0",
        },
        "paths": paths,
    })
}

/// Describe a data collection endpoint as an OpenAPI document.
///
/// The document covers `GET /.../data` and includes the collection's query
/// parameters plus an example payload containing the current metadata entries.
pub fn data_collection_docs(collection_path: &str, items: &[Metadata]) -> Value {
    build_openapi_doc(
        &format!("Data collection {collection_path}"),
        collection_path,
        json!({
            "get": {
                "summary": "List the data resources of the entity",
                "parameters": list_query_parameters(),
                "responses": {
                    "200": {
                        "description": "The data resources currently exposed by the entity.",
                        "content": {
                            "application/json": {
                                "schema": data_list_response_schema(),
                                "example": {
                                    "items": items,
                                },
                            },
                        },
                    },
                    "404": { "description": "The entity was not found." },
                },
            }
        }),
    )
}

/// Document the `GET /.../data` query parameters using ISO-style repeated keys.
fn list_query_parameters() -> Value {
    json!([
        {
            "name": "groups",
            "in": "query",
            "required": false,
            "description": "Filter by data group. Repeat the parameter to select multiple groups.",
            "style": "form",
            "explode": true,
            "schema": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        {
            "name": "categories",
            "in": "query",
            "required": false,
            "description": "Filter by data category. Repeat the parameter to select multiple categories.",
            "style": "form",
            "explode": true,
            "schema": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        {
            "name": "tags",
            "in": "query",
            "required": false,
            "description": "Filter by tag. Repeat the parameter to select multiple tags.",
            "style": "form",
            "explode": true,
            "schema": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        {
            "name": "include-schema",
            "in": "query",
            "required": false,
            "description": "Include the JSON schema of the data list response in the response body.",
            "schema": { "type": "boolean", "default": false }
        }
    ])
}

/// Schema for the `/data` response envelope.
fn data_list_response_schema() -> Value {
    let mut schema = DataList::schema();
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        properties.insert(
            "schema".to_owned(),
            json!({
                "type": "object",
                "description": "Optional JSON Schema for the data list response when include-schema=true."
            }),
        );
    }
    schema
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_collection_documents_get_with_filters_and_example_items() {
        let doc = data_collection_docs(
            "/components/Engine/data",
            &[Metadata {
                id: "rpm".into(),
                name: "rpm".into(),
                category: opensovd_models::data::DataCategory::CurrentData,
                translation_id: None,
                groups: Some(vec!["powertrain".into()]),
                tags: Some(vec!["OBD".into()]),
            }],
        );
        let path = &doc["paths"]["/components/Engine/data"];

        assert!(path["get"].is_object(), "GET must be documented");
        assert!(path["put"].is_null(), "PUT must not be documented");
        assert_eq!(path["get"]["parameters"][0]["name"], "groups");
        assert_eq!(path["get"]["parameters"][0]["explode"], true);
        assert_eq!(
            path["get"]["responses"]["200"]["content"]["application/json"]["example"]["items"][0]
                ["id"],
            "rpm"
        );
    }
}
