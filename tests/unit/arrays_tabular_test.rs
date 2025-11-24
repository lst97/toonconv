use toonconv::conversion::ConversionConfig;
use toonconv::convert_json_string;

#[test]
fn test_uniform_object_array_tabular() {
    let config = ConversionConfig::default();
    let json_str = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;

    let res = convert_json_string(json_str, &config).unwrap();
    let toon = res.content;

    // Expect tabular format header for uniform object array
    assert!(toon.contains("[2,]{id,name}:") || toon.contains("[2,]{name,id}:"));
}

#[test]
fn test_non_uniform_object_array_not_tabular() {
    let config = ConversionConfig::default();
    let json_str = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "username": "Bob"}]"#;

    let res = convert_json_string(json_str, &config).unwrap();
    let toon = res.content;

    // Should not use tabular format because keys differ
    assert!(!toon.contains("{id,name}") && !toon.contains("{name,id}"));
}
