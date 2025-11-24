//! Integration tests for complex JSON conversion scenarios
//!
//! End-to-end tests for real-world complex structures including
//! deeply nested objects, mixed arrays, and edge cases.

use serde_json::json;
use std::fs;
use std::path::PathBuf;
use toonconv::conversion::{convert_json_to_toon, ConversionConfig, QuoteStrategy};
use toonconv::parser::JsonSource;

#[test]
fn test_complex_nested_user_data() {
    let config = ConversionConfig::default();

    let json = json!({
        "user": {
            "id": 12345,
            "username": "alice_dev",
            "email": "alice@example.com",
            "profile": {
                "firstName": "Alice",
                "lastName": "Developer",
                "avatar": "https://example.com/avatar.jpg",
                "bio": "Full-stack developer with 10+ years of experience",
                "location": {
                    "city": "San Francisco",
                    "state": "CA",
                    "country": "USA",
                    "coordinates": {
                        "lat": 37.7749,
                        "lng": -122.4194
                    }
                },
                "social": {
                    "twitter": "@alice_dev",
                    "github": "alice-dev",
                    "linkedin": "alice-developer"
                }
            },
            "settings": {
                "notifications": {
                    "email": true,
                    "push": false,
                    "sms": false
                },
                "privacy": {
                    "profileVisibility": "public",
                    "showEmail": false,
                    "showLocation": true
                },
                "preferences": {
                    "theme": "dark",
                    "language": "en-US",
                    "timezone": "America/Los_Angeles"
                }
            },
            "activity": {
                "lastLogin": "2025-11-19T10:30:00Z",
                "loginCount": 1523,
                "posts": [
                    {
                        "id": 1,
                        "title": "Getting Started with Rust",
                        "tags": ["rust", "programming", "tutorial"],
                        "views": 1205,
                        "likes": 89
                    },
                    {
                        "id": 2,
                        "title": "Advanced TOON Format",
                        "tags": ["toon", "json", "optimization"],
                        "views": 734,
                        "likes": 56
                    }
                ]
            }
        }
    });

    let source = JsonSource::Value(json.clone());
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Verify all nested fields are present
    assert!(result.content.contains("alice_dev"));
    assert!(result.content.contains("alice@example.com"));
    assert!(result.content.contains("San Francisco"));
    assert!(result.content.contains("37.7749"));
    assert!(result.content.contains("Getting Started with Rust"));
    assert!(result.content.contains("dark"));

    // Verify data integrity
    assert!(result.statistics.as_ref().unwrap().data_integrity_verified);
}

#[test]
fn test_ecommerce_order_structure() {
    let config = ConversionConfig::default();

    let json = json!({
        "order": {
            "id": "ORD-2025-001234",
            "customerId": 54321,
            "status": "shipped",
            "createdAt": "2025-11-15T14:22:00Z",
            "updatedAt": "2025-11-18T09:15:00Z",
            "items": [
                {
                    "productId": "PROD-001",
                    "name": "Wireless Headphones",
                    "quantity": 1,
                    "price": 79.99,
                    "discount": 10.0,
                    "subtotal": 69.99
                },
                {
                    "productId": "PROD-002",
                    "name": "USB-C Cable",
                    "quantity": 3,
                    "price": 12.99,
                    "discount": 0.0,
                    "subtotal": 38.97
                }
            ],
            "shipping": {
                "method": "standard",
                "cost": 5.99,
                "address": {
                    "street": "123 Main St",
                    "city": "New York",
                    "state": "NY",
                    "zip": "10001",
                    "country": "USA"
                },
                "tracking": {
                    "carrier": "USPS",
                    "number": "9400111899223456789012",
                    "estimatedDelivery": "2025-11-20"
                }
            },
            "payment": {
                "method": "credit_card",
                "last4": "4242",
                "transactionId": "txn_abc123xyz",
                "status": "completed"
            },
            "totals": {
                "subtotal": 108.96,
                "shipping": 5.99,
                "tax": 9.08,
                "total": 124.03
            }
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Verify key order data
    assert!(result.content.contains("ORD-2025-001234"));
    assert!(result.content.contains("Wireless Headphones"));
    assert!(result.content.contains("123 Main St"));
    assert!(result.content.contains("124.03"));

    // Verify arrays are formatted
    assert!(result.content.contains("items"));
    assert!(result.content.contains("PROD-001"));
}

#[test]
fn test_api_response_with_pagination() {
    let config = ConversionConfig::default();

    let json = json!({
        "data": (0..25).map(|i| json!({
            "id": i,
            "title": format!("Article {}", i),
            "author": format!("Author {}", i % 5),
            "published": i % 2 == 0
        })).collect::<Vec<_>>(),
        "pagination": {
            "page": 1,
            "perPage": 25,
            "total": 1000,
            "totalPages": 40,
            "hasNext": true,
            "hasPrev": false
        },
        "meta": {
            "apiVersion": "2.0",
            "timestamp": "2025-11-19T10:30:00Z",
            "requestId": "req_xyz789"
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("pagination"));
    assert!(result.content.contains("total"));
    assert!(result.content.contains("1000"));
}

#[test]
fn test_deeply_nested_config_structure() {
    let config = ConversionConfig::default();

    let json = json!({
        "application": {
            "name": "MyApp",
            "version": "1.0.0",
            "server": {
                "host": "localhost",
                "port": 8080,
                "ssl": {
                    "enabled": true,
                    "cert": "/path/to/cert.pem",
                    "key": "/path/to/key.pem"
                }
            },
            "database": {
                "primary": {
                    "host": "db1.example.com",
                    "port": 5432,
                    "name": "myapp_prod",
                    "pool": {
                        "min": 5,
                        "max": 20,
                        "idleTimeout": 30000
                    }
                },
                "replica": {
                    "host": "db2.example.com",
                    "port": 5432,
                    "name": "myapp_prod",
                    "readOnly": true
                }
            },
            "cache": {
                "redis": {
                    "host": "redis.example.com",
                    "port": 6379,
                    "db": 0,
                    "ttl": {
                        "default": 3600,
                        "sessions": 86400,
                        "api": 300
                    }
                }
            }
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("MyApp"));
    assert!(result.content.contains("localhost"));
    assert!(result.content.contains("db1.example.com"));
    assert!(result.content.contains("redis.example.com"));
}

#[test]
fn test_mixed_type_array_scenarios() {
    let config = ConversionConfig::default();

    let json = json!({
        "mixedData": [
            42,
            "hello",
            true,
            null,
            {"type": "object", "value": 100},
            [1, 2, 3],
            3.14,
            false,
            {"type": "another", "data": "test"}
        ]
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Verify all types are present
    assert!(result.content.contains("42"));
    assert!(result.content.contains("hello"));
    assert!(result.content.contains("true"));
    assert!(result.content.contains("false"));
    assert!(result.content.contains("null"));
    assert!(result.content.contains("3.14"));
    assert!(result.content.contains("type"));
}

#[test]
fn test_unicode_and_special_characters() {
    let mut config = ConversionConfig::default();
    config.quote_strings = QuoteStrategy::Smart;

    let json = json!({
        "languages": {
            "english": "Hello, World!",
            "chinese": "你好世界",
            "japanese": "こんにちは世界",
            "arabic": "مرحبا بالعالم",
            "emoji": "😀🎉🚀💻",
            "mixed": "Hello 世界 🌍"
        },
        "specialChars": {
            "quotes": "She said \"hello\"",
            "newlines": "line1\nline2\nline3",
            "tabs": "col1\tcol2\tcol3",
            "backslash": "C:\\Users\\Alice",
            "control": "Special chars: :,{}[]"
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Verify unicode preservation
    assert!(result.content.contains("你好世界"));
    assert!(result.content.contains("😀"));

    // Verify escaping
    assert!(result.content.contains("\\n") || result.content.contains("\n"));
    assert!(result.content.contains("\\\""));
}

#[test]
fn test_empty_and_null_scenarios() {
    let config = ConversionConfig::default();

    let json = json!({
        "emptyString": "",
        "emptyObject": {},
        "emptyArray": [],
        "nullValue": null,
        "nested": {
            "emptyNested": {},
            "arrayWithEmpty": [{}, [], null, ""],
            "nullField": null
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("{}"));
    assert!(result.content.contains("[]"));
    assert!(result.content.contains("null"));
    assert!(result.content.contains("\"\""));
}

#[test]
fn test_large_uniform_array_tabular() {
    let config = ConversionConfig::default();

    let users: Vec<_> = (0..100)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("User{}", i),
                "email": format!("user{}@example.com", i),
                "age": 20 + (i % 50),
                "active": i % 2 == 0
            })
        })
        .collect();

    let json = json!({"users": users});
    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Should use tabular format for uniform array
    assert!(result.content.contains("[100"));
    assert!(result.content.contains("users"));
}

#[test]
fn test_file_based_complex_conversion() {
    let config = ConversionConfig::default();

    let json = json!({
        "testData": {
            "nested": {
                "values": [1, 2, 3, 4, 5]
            }
        }
    });

    // Create temp files
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_complex_input.json");
    let output_path = temp_dir.join("test_complex_output.toon");

    // Write input
    fs::write(&input_path, json.to_string()).unwrap();

    // Convert
    let source = JsonSource::File(input_path.clone());
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Write output
    fs::write(&output_path, &result.content).unwrap();

    // Verify output exists and has content
    assert!(output_path.exists());
    let output_content = fs::read_to_string(&output_path).unwrap();
    assert!(!output_content.is_empty());
    assert!(output_content.contains("testData"));

    // Cleanup
    fs::remove_file(input_path).ok();
    fs::remove_file(output_path).ok();
}

#[test]
fn test_complex_structure_with_validation() {
    let mut config = ConversionConfig::default();
    config.validate_output = true;

    let json = json!({
        "data": {
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        }
    });

    let source = JsonSource::Value(json.clone());
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Validation should pass
    assert!(result.statistics.as_ref().unwrap().data_integrity_verified);

    // All data should be present
    assert!(result.content.contains("Alice"));
    assert!(result.content.contains("Bob"));
    assert!(result.content.contains("dark"));
}

#[test]
fn test_real_world_json_api_response() {
    let config = ConversionConfig::default();

    // Simulates a GitHub API response structure
    let json = json!({
        "repository": {
            "name": "toonconv",
            "full_name": "user/toonconv",
            "description": "JSON to TOON converter",
            "private": false,
            "owner": {
                "login": "user",
                "id": 12345,
                "avatar_url": "https://avatars.example.com/u/12345",
                "type": "User"
            },
            "html_url": "https://github.com/user/toonconv",
            "created_at": "2025-11-01T10:00:00Z",
            "updated_at": "2025-11-19T15:30:00Z",
            "pushed_at": "2025-11-19T15:30:00Z",
            "size": 1234,
            "stargazers_count": 42,
            "watchers_count": 15,
            "language": "Rust",
            "has_issues": true,
            "has_projects": true,
            "has_downloads": true,
            "has_wiki": true,
            "forks_count": 8,
            "open_issues_count": 3,
            "default_branch": "main",
            "topics": ["rust", "json", "toon", "converter"]
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("toonconv"));
    assert!(result.content.contains("Rust"));
    assert!(result.content.contains("42"));
}
