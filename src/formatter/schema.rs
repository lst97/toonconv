//! Schema declaration generation for TOON arrays
//!
//! Generates schema information for uniform arrays to enable
//! compact tabular formatting with field declarations.

use crate::error::FormattingResult;
use serde_json::Value;
use std::collections::HashSet;

/// Schema generator for TOON arrays
pub struct SchemaGenerator {
    /// Include schema declarations in output
    include_schema: bool,
}

impl SchemaGenerator {
    /// Create a new schema generator
    pub fn new(include_schema: bool) -> Self {
        Self { include_schema }
    }

    /// Generate schema declaration for an array
    pub fn generate_schema(&self, array: &[Value]) -> FormattingResult<Option<ArraySchema>> {
        if !self.include_schema || array.is_empty() {
            return Ok(None);
        }

        // Detect array type
        if self.is_uniform_object_array(array) {
            Ok(Some(self.generate_object_array_schema(array)?))
        } else if self.is_primitive_array(array) {
            Ok(Some(self.generate_primitive_array_schema(array)?))
        } else {
            // Mixed or complex array - no schema
            Ok(None)
        }
    }

    /// Check if array contains uniform objects
    fn is_uniform_object_array(&self, array: &[Value]) -> bool {
        if array.is_empty() {
            return false;
        }

        // All elements must be objects
        if !array.iter().all(|v| v.is_object()) {
            return false;
        }

        // Get keys from first object
        let first_obj = array[0].as_object().unwrap();
        let first_keys: HashSet<&str> = first_obj.keys().map(|k| k.as_str()).collect();

        // All objects must have identical keys
        array.iter().all(|v| {
            let obj = v.as_object().unwrap();
            let keys: HashSet<&str> = obj.keys().map(|k| k.as_str()).collect();
            keys == first_keys
        })
    }

    /// Check if array contains primitives only
    fn is_primitive_array(&self, array: &[Value]) -> bool {
        if array.is_empty() {
            return false;
        }

        array
            .iter()
            .all(|v| v.is_null() || v.is_boolean() || v.is_number() || v.is_string())
    }

    /// Generate schema for uniform object array
    fn generate_object_array_schema(&self, array: &[Value]) -> FormattingResult<ArraySchema> {
        let first_obj = array[0].as_object().unwrap();
        let mut fields: Vec<String> = first_obj.keys().map(|k| k.to_string()).collect();

        // Sort fields for consistency
        fields.sort();

        // Detect field types
        let mut field_types = Vec::new();
        for field in &fields {
            let field_type = self.detect_field_type(array, field)?;
            field_types.push(field_type);
        }

        Ok(ArraySchema {
            schema_type: SchemaType::UniformObject,
            length: array.len(),
            fields: Some(fields),
            field_types: Some(field_types),
            element_type: None,
        })
    }

    /// Generate schema for primitive array
    fn generate_primitive_array_schema(&self, array: &[Value]) -> FormattingResult<ArraySchema> {
        let element_type = self.detect_primitive_type(&array[0])?;

        // Verify all elements have same type (or compatible types)
        for value in array {
            let value_type = self.detect_primitive_type(value)?;
            if !self.types_compatible(&element_type, &value_type) {
                // Not uniform - no schema
                return Ok(ArraySchema {
                    schema_type: SchemaType::Mixed,
                    length: array.len(),
                    fields: None,
                    field_types: None,
                    element_type: None,
                });
            }
        }

        Ok(ArraySchema {
            schema_type: SchemaType::Primitive,
            length: array.len(),
            fields: None,
            field_types: None,
            element_type: Some(element_type),
        })
    }

    /// Detect the type of a field across all array elements
    fn detect_field_type(&self, array: &[Value], field: &str) -> FormattingResult<FieldType> {
        let mut null_count = 0;
        let mut detected_type: Option<FieldType> = None;

        for value in array {
            let obj = value.as_object().unwrap();
            if let Some(field_value) = obj.get(field) {
                if field_value.is_null() {
                    null_count += 1;
                    continue;
                }

                let value_type = self.value_to_field_type(field_value)?;

                if let Some(ref existing_type) = detected_type {
                    if existing_type != &value_type {
                        // Mixed types in same field
                        return Ok(FieldType::Mixed);
                    }
                } else {
                    detected_type = Some(value_type);
                }
            }
        }

        // If more than 10% are null, mark as nullable
        let nullable = (null_count as f64 / array.len() as f64) > 0.1;

        match detected_type {
            Some(mut field_type) => {
                if nullable {
                    field_type = FieldType::Nullable(Box::new(field_type));
                }
                Ok(field_type)
            }
            None => Ok(FieldType::Null),
        }
    }

    /// Convert JSON value to field type
    fn value_to_field_type(&self, value: &Value) -> FormattingResult<FieldType> {
        Ok(match value {
            Value::Null => FieldType::Null,
            Value::Bool(_) => FieldType::Boolean,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    FieldType::Integer
                } else {
                    FieldType::Float
                }
            }
            Value::String(_) => FieldType::String,
            Value::Array(_) => FieldType::Array,
            Value::Object(_) => FieldType::Object,
        })
    }

    /// Detect primitive type
    fn detect_primitive_type(&self, value: &Value) -> FormattingResult<FieldType> {
        self.value_to_field_type(value)
    }

    /// Check if two types are compatible (e.g., int and float)
    fn types_compatible(&self, type1: &FieldType, type2: &FieldType) -> bool {
        match (type1, type2) {
            (FieldType::Integer, FieldType::Float) => true,
            (FieldType::Float, FieldType::Integer) => true,
            (a, b) => a == b,
        }
    }

    /// Format schema declaration as TOON string
    pub fn format_schema(&self, schema: &ArraySchema) -> String {
        match schema.schema_type {
            SchemaType::UniformObject => {
                if let Some(ref fields) = schema.fields {
                    format!("[{},]{{{}}}", schema.length, fields.join(","))
                } else {
                    format!("[{}]", schema.length)
                }
            }
            SchemaType::Primitive => {
                format!("[{}]", schema.length)
            }
            SchemaType::Mixed => {
                format!("[{}]", schema.length)
            }
        }
    }
}

/// Array schema information
#[derive(Debug, Clone)]
pub struct ArraySchema {
    /// Type of schema
    pub schema_type: SchemaType,

    /// Number of elements in array
    pub length: usize,

    /// Field names (for object arrays)
    pub fields: Option<Vec<String>>,

    /// Field types (for object arrays)
    pub field_types: Option<Vec<FieldType>>,

    /// Element type (for primitive arrays)
    pub element_type: Option<FieldType>,
}

/// Schema type classification
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    UniformObject,
    Primitive,
    Mixed,
}

/// Field type information
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Object,
    Mixed,
    Nullable(Box<FieldType>),
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Null => write!(f, "null"),
            FieldType::Boolean => write!(f, "bool"),
            FieldType::Integer => write!(f, "int"),
            FieldType::Float => write!(f, "float"),
            FieldType::String => write!(f, "string"),
            FieldType::Array => write!(f, "array"),
            FieldType::Object => write!(f, "object"),
            FieldType::Mixed => write!(f, "mixed"),
            FieldType::Nullable(inner) => write!(f, "{}?", inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_uniform_object_array_schema() {
        let generator = SchemaGenerator::new(true);

        let array = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
        ];

        let schema = generator.generate_schema(&array).unwrap().unwrap();
        assert_eq!(schema.schema_type, SchemaType::UniformObject);
        assert_eq!(schema.length, 2);
        assert!(schema.fields.is_some());

        let fields = schema.fields.unwrap();
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"age".to_string()));
    }

    #[test]
    fn test_primitive_array_schema() {
        let generator = SchemaGenerator::new(true);

        let array = vec![json!(1), json!(2), json!(3)];
        let schema = generator.generate_schema(&array).unwrap().unwrap();

        assert_eq!(schema.schema_type, SchemaType::Primitive);
        assert_eq!(schema.length, 3);
        assert_eq!(schema.element_type, Some(FieldType::Integer));
    }

    #[test]
    fn test_mixed_array_no_schema() {
        let generator = SchemaGenerator::new(true);

        let array = vec![json!(1), json!("hello"), json!(true)];
        let schema = generator.generate_schema(&array).unwrap();

        // Mixed arrays may return None or Mixed schema
        if let Some(s) = schema {
            assert_eq!(s.schema_type, SchemaType::Mixed);
        }
    }

    #[test]
    fn test_schema_disabled() {
        let generator = SchemaGenerator::new(false);

        let array = vec![json!({"a": 1}), json!({"a": 2})];
        let schema = generator.generate_schema(&array).unwrap();

        assert!(schema.is_none());
    }

    #[test]
    fn test_format_object_schema() {
        let generator = SchemaGenerator::new(true);

        let schema = ArraySchema {
            schema_type: SchemaType::UniformObject,
            length: 2,
            fields: Some(vec!["age".to_string(), "name".to_string()]),
            field_types: None,
            element_type: None,
        };

        let formatted = generator.format_schema(&schema);
        assert_eq!(formatted, "[2,]{age,name}");
    }

    #[test]
    fn test_format_primitive_schema() {
        let generator = SchemaGenerator::new(true);

        let schema = ArraySchema {
            schema_type: SchemaType::Primitive,
            length: 5,
            fields: None,
            field_types: None,
            element_type: Some(FieldType::Integer),
        };

        let formatted = generator.format_schema(&schema);
        assert_eq!(formatted, "[5]");
    }

    #[test]
    fn test_nullable_field_detection() {
        let generator = SchemaGenerator::new(true);

        let array = vec![
            json!({"value": 10}),
            json!({"value": null}),
            json!({"value": 20}),
        ];

        let schema = generator.generate_schema(&array).unwrap().unwrap();
        let field_types = schema.field_types.unwrap();

        // With 33% nulls, should be marked nullable (threshold is 10%)
        assert!(matches!(field_types[0], FieldType::Nullable(_)));
    }

    #[test]
    fn test_non_uniform_objects() {
        let generator = SchemaGenerator::new(true);

        let array = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "score": 95}), // Different keys
        ];

        let schema = generator.generate_schema(&array).unwrap();
        // Non-uniform arrays should not get object schema
        assert!(schema.is_none() || schema.unwrap().schema_type != SchemaType::UniformObject);
    }

    #[test]
    fn test_empty_array() {
        let generator = SchemaGenerator::new(true);
        let array: Vec<Value> = vec![];
        let schema = generator.generate_schema(&array).unwrap();
        assert!(schema.is_none());
    }

    #[test]
    fn test_field_type_display() {
        assert_eq!(FieldType::Integer.to_string(), "int");
        assert_eq!(FieldType::String.to_string(), "string");
        assert_eq!(
            FieldType::Nullable(Box::new(FieldType::Integer)).to_string(),
            "int?"
        );
    }
}
