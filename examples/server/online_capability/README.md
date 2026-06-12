# Online Capability Description Example

SOVD **online capability description**
ISO 17978-3: appending `/docs` to any data resource path returns
a self-contained OpenAPI 3.1 specification describing how to interact with that
resource — its supported HTTP methods, request/response schemas, and status
codes — without needing an overall offline capability description.

## Topology

```text
SOVDServer
  └── Component: "engine"
        └── Data Provider
              └── rpm  (read-write, CurrentData)
```

## Running

```bash
cargo run -p opensovd-examples-server --example online_capability
```

The server starts on `http://127.0.0.1:7691`.

## Example Requests

```bash
# Read the data resource value
curl -s http://localhost:7691/sovd/v1/components/engine/data/rpm | jq

# Write a new value
curl -s -X PUT http://localhost:7691/sovd/v1/components/engine/data/rpm \
  -H 'Content-Type: application/json' \
  -d '{"data": {"value": 1500.0}}'

# Retrieve the ONLINE CAPABILITY DESCRIPTION for the resource
curl -s http://localhost:7691/sovd/v1/components/engine/data/rpm/docs | jq
```

The `/docs` response is a valid OpenAPI 3.1 document. It documents the methods
the resource actually supports (`GET` and `PUT` here, since `rpm` is
read-write), the `include-schema` query parameter, the `200` / `204` / `404`
status codes, and the request/response payload schemas.

Notably, the `data` field of each payload carries the resource's **own value
schema**, sourced dynamically from the data provider — so a client sees that
`rpm` reads and writes `{ "value": <number> }`, not just an opaque object
(abbreviated below):

```json
{
  "openapi": "3.1.0",
  "info": { "title": "Data resource /components/engine/data/rpm", "version": "1.0.0" },
  "paths": {
    "/components/engine/data/rpm": {
      "get": {
        "summary": "Read the value of the data resource",
        "parameters": [
          { "name": "include-schema", "in": "query", "schema": { "type": "boolean" } }
        ],
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "id": { "type": "string" },
                    "data": {
                      "type": "object",
                      "properties": { "value": { "type": "number", "format": "double" } },
                      "required": ["value"]
                    }
                  },
                  "required": ["id", "data"]
                }
              }
            }
          },
          "404": {}
        }
      },
      "put": {
        "summary": "Write the value of the data resource",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "data": {
                    "type": "object",
                    "properties": { "value": { "type": "number", "format": "double" } },
                    "required": ["value"]
                  }
                },
                "required": ["data"]
              }
            }
          }
        },
        "responses": { "204": {}, "404": {} }
      }
    }
  }
}
```

> A read-only resource omits the `put` entry entirely, and a resource whose
> provider supplies no schema falls back to an opaque `data` object.
