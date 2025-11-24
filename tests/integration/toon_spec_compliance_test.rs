//! TOON specification compliance tests
//!
//! Tests to ensure output matches the official TOON format specification

use serde_json::json;
use toonconv::conversion::config::ConversionConfig;
use toonconv::conversion::convert_json_to_toon;

#[test]
fn test_basic_tabular_array() {
    let json = json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    });

    let config = ConversionConfig::default();
    let result = convert_json_to_toon(&json, &config).unwrap();

    let expected = "users[2]{id,name}:\n  1,Alice\n  2,Bob";
    assert_eq!(result.content.trim(), expected);
}

#[test]
fn test_nested_object_with_tabular_array() {
    let json = json!({
        "context": {
            "app": "SalesDB",
            "version": 1.2
        },
        "products": [
            {"id": "A100", "price": 25.50, "stock": 10},
            {"id": "B200", "price": 9.99, "stock": 0},
            {"id": "C300", "price": 120.00, "stock": 5}
        ]
    });

    let config = ConversionConfig::default();
    let result = convert_json_to_toon(&json, &config).unwrap();

    // TOON spec: numbers without trailing zeros
    let expected = r#"context:
  app: SalesDB
  version: 1.2
products[3]{id,price,stock}:
  A100,25.5,10
  B200,9.99,0
  C300,120,5"#;

    assert_eq!(result.content.trim(), expected);
}

#[test]
fn test_api_endpoints_with_mixed_arrays() {
    let json = json!({
        "api": {
            "version": "2.1.0",
            "endpoints": [
                {
                    "path": "/users",
                    "methods": ["GET", "POST"],
                    "authRequired": true
                },
                {
                    "path": "/products",
                    "methods": ["GET", "POST", "PUT", "DELETE"],
                    "authRequired": true
                },
                {
                    "path": "/health",
                    "methods": ["GET"],
                    "authRequired": false
                }
            ],
            "rateLimit": {
                "requestsPerMinute": 100,
                "burstSize": 20
            },
            "status": "operational"
        }
    });

    let config = ConversionConfig::default();
    let result = convert_json_to_toon(&json, &config).unwrap();

    // TOON spec: objects in list format have dash then newline, then indented content
    let expected = r#"api:
  version: 2.1.0
  endpoints[3]:
    -
      path: /users
      methods[2]: GET,POST
      authRequired: true
    -
      path: /products
      methods[4]: GET,POST,PUT,DELETE
      authRequired: true
    -
      path: /health
      methods[1]: GET
      authRequired: false
  rateLimit:
    requestsPerMinute: 100
    burstSize: 20
  status: operational"#;

    assert_eq!(result.content.trim(), expected);
}

#[test]
fn test_primitive_array_inline() {
    let json = json!({
        "tags": ["rust", "programming", "web"]
    });

    let config = ConversionConfig::default();
    let result = convert_json_to_toon(&json, &config).unwrap();

    let expected = "tags[3]: rust,programming,web";
    assert_eq!(result.content.trim(), expected);
}

#[test]
fn test_no_quotes_on_simple_strings() {
    let json = json!({
        "name": "Alice",
        "city": "New York",
        "status": "active"
    });

    let config = ConversionConfig::default();
    let result = convert_json_to_toon(&json, &config).unwrap();

    // Should not have quotes around simple strings
    assert!(!result.content.contains('"'));
    assert!(result.content.contains("name: Alice"));
    assert!(result.content.contains("city: New York"));
    assert!(result.content.contains("status: active"));
}
