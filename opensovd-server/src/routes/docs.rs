// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

//! SOVD online capability descriptions.
//!
//! `GET /{path}/docs` returns a self-contained OpenAPI 3.1 document for a single
//! resource: its methods, payload schemas and status codes. A client can use it
//! without an offline capability description for the whole vehicle.
//!
//! Only the data resource is wired up so far; see [`data_resource_docs`].

use opensovd_models::data::{ReadResponse, WriteRequest};
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

/// Describe a single data resource as an OpenAPI document.
///
/// `GET` is included when `readable`, `PUT` when `writable`. A `resource_schema`,
/// if present, is inlined into the payload `data` field so the doc shows the real
/// value shape instead of an opaque object.
pub fn data_resource_docs(
    resource_path: &str,
    readable: bool,
    writable: bool,
    resource_schema: Option<&Value>,
) -> Value {
    let mut path_item = serde_json::Map::new();

    if readable {
        path_item.insert("get".to_owned(), read_operation(resource_schema));
    }
    if writable {
        path_item.insert("put".to_owned(), write_operation(resource_schema));
    }

    build_openapi_doc(
        &format!("Data resource {resource_path}"),
        resource_path,
        Value::Object(path_item),
    )
}

/// Splice the resource's value schema into the `data` field of an envelope schema
/// ([`ReadResponse`] / [`WriteRequest`]). Returns the envelope untouched when
/// there is no schema to splice in.
fn with_data_schema(mut envelope: Value, resource_schema: Option<&Value>) -> Value {
    let Some(schema) = resource_schema else {
        return envelope;
    };
    if let Some(data) = envelope.pointer_mut("/properties/data") {
        let mut schema = schema.clone();
        // $schema belongs on a document root, not on an inlined subschema.
        if let Some(obj) = schema.as_object_mut() {
            obj.remove("$schema");
        }
        *data = schema;
    }
    envelope
}

/// The `GET` (read) operation.
fn read_operation(resource_schema: Option<&Value>) -> Value {
    let response_schema = with_data_schema(ReadResponse::schema(), resource_schema);
    json!({
        "summary": "Read the value of the data resource",
        "parameters": [
            {
                "name": "include-schema",
                "in": "query",
                "required": false,
                "description": "Include the JSON schema of the value in the response.",
                "schema": { "type": "boolean", "default": false },
            }
        ],
        "responses": {
            "200": {
                "description": "The current value of the data resource.",
                "content": {
                    "application/json": { "schema": response_schema },
                },
            },
            "404": { "description": "The entity or data resource was not found." },
        },
    })
}

/// The `PUT` (write) operation.
fn write_operation(resource_schema: Option<&Value>) -> Value {
    let request_schema = with_data_schema(WriteRequest::schema(), resource_schema);
    json!({
        "summary": "Write the value of the data resource",
        "requestBody": {
            "required": true,
            "content": {
                "application/json": { "schema": request_schema },
            },
        },
        "responses": {
            "204": { "description": "The value was written successfully." },
            "404": { "description": "The entity or data resource was not found." },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_schema() -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": { "value": { "type": "number" } },
            "required": ["value"],
        })
    }

    #[test]
    fn read_write_resource_documents_both_methods() {
        let schema = value_schema();
        let doc = data_resource_docs("/components/Engine/data/Rpm", true, true, Some(&schema));

        assert_eq!(doc["openapi"], "3.1.0");
        assert_eq!(
            doc["info"]["title"],
            "Data resource /components/Engine/data/Rpm"
        );

        let path = &doc["paths"]["/components/Engine/data/Rpm"];
        assert!(path["get"].is_object(), "GET method must be documented");
        assert!(path["put"].is_object(), "PUT method must be documented");

        assert!(
            path["get"]["responses"]["200"]["content"]["application/json"]["schema"].is_object()
        );
        assert!(path["put"]["requestBody"]["content"]["application/json"]["schema"].is_object());

        assert_eq!(path["get"]["parameters"][0]["name"], "include-schema");
    }

    #[test]
    fn resource_schema_is_inlined_into_the_data_field() {
        let schema = value_schema();
        let doc = data_resource_docs("/components/Engine/data/Rpm", true, true, Some(&schema));
        let path = &doc["paths"]["/components/Engine/data/Rpm"];

        // The resource's own value schema ends up under data on the read side...
        let read_data = &path["get"]["responses"]["200"]["content"]["application/json"]["schema"]["properties"]
            ["data"];
        assert_eq!(read_data["properties"]["value"]["type"], "number");
        assert!(read_data.get("$schema").is_none());

        // ...and on the write side.
        let write_data = &path["put"]["requestBody"]["content"]["application/json"]["schema"]["properties"]
            ["data"];
        assert_eq!(write_data["properties"]["value"]["type"], "number");
    }

    #[test]
    fn without_resource_schema_data_stays_opaque() {
        let doc = data_resource_docs("/components/Engine/data/Rpm", true, false, None);
        let path = &doc["paths"]["/components/Engine/data/Rpm"];

        // No schema from the provider, so data keeps the envelope's opaque shape.
        let read_data = &path["get"]["responses"]["200"]["content"]["application/json"]["schema"]["properties"]
            ["data"];
        assert_eq!(read_data["properties"]["value"]["type"], Value::Null);
    }

    #[test]
    fn read_only_resource_documents_get_but_not_put() {
        let schema = value_schema();
        let doc = data_resource_docs("/components/Engine/data/Rpm", true, false, Some(&schema));
        let path = &doc["paths"]["/components/Engine/data/Rpm"];

        assert!(path["get"].is_object(), "GET must be documented");
        assert!(path["put"].is_null(), "PUT must not be documented");
    }

    #[test]
    fn write_only_resource_documents_put_but_not_get() {
        let schema = value_schema();
        let doc = data_resource_docs("/components/Engine/data/Rpm", false, true, Some(&schema));
        let path = &doc["paths"]["/components/Engine/data/Rpm"];

        assert!(path["put"].is_object(), "PUT must be documented");
        assert!(path["get"].is_null(), "GET must not be documented");
    }
}
