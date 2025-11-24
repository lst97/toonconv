//! Integration tests for stdin-to-TOON conversion workflow
//! 
//! These tests simulate the full end-to-end workflow:
//! - Reading JSON from stdin
//! - Converting to TOON format
//! - Outputting to terminal
//! - Error handling for various scenarios

#[cfg(test)]
mod stdin_conversion_tests {
    use std::process::{Command, Stdio};
    use std::io::{Write, BufRead, BufReader};
    use std::time::Duration;

    fn run_toonconv_stdin(input: &str, args: &[&str]) -> Result<(String, String), String> {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", "toonconv", "--"])
           .args(args)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to start process: {}", e))?;
        
        // Send input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }
        
        let stdout = child.stdout.take()
            .map(|mut stdout| {
                let mut reader = BufReader::new(stdout);
                let mut output = String::new();
                reader.read_to_string(&mut output).unwrap();
                output
            }).unwrap_or_default();
        
        let stderr = child.stderr.take()
            .map(|mut stderr| {
                let mut reader = BufReader::new(stderr);
                let mut error = String::new();
                reader.read_to_string(&mut error).unwrap();
                error
            }).unwrap_or_default();
        
        let status = child.wait_with_output()
            .map_err(|e| format!("Failed to wait for process: {}", e))?;
        
        Ok((stdout, stderr))
    }

    /// Test basic stdin conversion workflow
    #[test]
    fn test_basic_stdin_conversion() {
        let input = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        // Should have successful conversion
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Verify TOON format
        assert!(stdout.contains("name:"));
        assert!(stdout.contains("Alice"));
        assert!(stdout.contains("age:"));
        assert!(stdout.contains("30"));
        assert!(stdout.contains("active:"));
        assert!(stdout.contains("true"));
        
        // Should not contain JSON syntax
        assert!(!stdout.contains("{"));
        assert!(!stdout.contains("}"));
        assert!(!stdout.contains("\""));
    }

    /// Test stdin with invalid JSON
    #[test]
    fn test_stdin_invalid_json() {
        let input = r#"{"name": "test", "value": }"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        // Should have error output
        assert!(!stderr.is_empty(), "Should have error message");
        assert!(stdout.is_empty() || stdout.trim().is_empty(), "Should have no TOON output");
        
        // Error should mention JSON parse error
        assert!(stderr.contains("JSON parse error") || stderr.contains("parse error"),
               "Should mention JSON parse error: {}", stderr);
    }

    /// Test stdin with empty input
    #[test]
    fn test_stdin_empty_input() {
        let input = "";
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        // Should have error for empty input
        assert!(!stderr.is_empty(), "Should have error message for empty input");
        assert!(stdout.is_empty() || stdout.trim().is_empty(), "Should have no TOON output");
        
        // Error should mention empty JSON
        assert!(stderr.contains("Empty") || stderr.contains("empty"),
               "Should mention empty input: {}", stderr);
    }

    /// Test stdin with whitespace-only input
    #[test]
    fn test_stdin_whitespace_input() {
        let input = "   \n\t  ";
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        // Should have error for empty input after trimming
        assert!(!stderr.is_empty(), "Should have error message for whitespace input");
        assert!(stdout.is_empty() || stdout.trim().is_empty(), "Should have no TOON output");
    }

    /// Test stdin with statistics flag
    #[test]
    fn test_stdin_with_stats() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin", "--stats"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have output");
        
        // Should contain conversion results
        assert!(stdout.contains("name:"));
        assert!(stdout.contains("Alice"));
        
        // Should contain statistics
        assert!(stdout.contains("Input size") || stdout.contains("output") || stdout.contains("statistics"),
               "Should show statistics: {}", stdout);
    }

    /// Test stdin with quiet flag
    #[test]
    fn test_stdin_with_quiet() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin", "--quiet"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should only have TOON content, no extra messages
        assert!(stdout.contains("name:"));
        assert!(stdout.contains("Alice"));
        
        // Should not have extra messages when quiet
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 1, "Should have only TOON output line in quiet mode");
    }

    /// Test stdin with custom delimiter
    #[test]
    fn test_stdin_with_custom_delimiter() {
        let input = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin", "--delimiter", "tab"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should use tab delimiter
        assert!(stdout.contains('\t'), "Should contain tab character");
        assert!(stdout.contains("[2,]{id,name}:"));
        assert!(stdout.contains("1\tAlice"));
        assert!(stdout.contains("2\tBob"));
    }

    /// Test stdin with custom indentation
    #[test]
    fn test_stdin_with_custom_indent() {
        let input = r#"{"user": {"name": "Alice", "settings": {"theme": "dark"}}}"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin", "--indent", "4"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should have proper indentation for nested structure
        assert!(stdout.contains("user:"));
        assert!(stdout.contains("settings:"));
        assert!(stdout.contains("name:"));
        assert!(stdout.contains("Alice"));
        
        // Check that nested content is indented
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(lines.len() >= 3, "Should have multiple lines for nested structure");
    }

    /// Test stdin with memory limit
    #[test]
    fn test_stdin_with_memory_limit() {
        let input = r#"{"data": "test"}"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin", "--memory-limit", "1MB"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        assert!(stdout.contains("data:"));
        assert!(stdout.contains("test"));
    }

    /// Test stdin array conversion
    #[test]
    fn test_stdin_array_conversion() {
        let input = r#"[1, 2, 3, 4, 5]"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should use primitive array format
        assert!(stdout.starts_with("[5]:"));
        assert!(stdout.contains("1,2,3,4,5"));
    }

    /// Test stdin user array conversion
    #[test]
    fn test_stdin_user_array_conversion() {
        let input = r#"[{"name": "Alice", "role": "admin"}, {"name": "Bob", "role": "user"}]"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should use tabular format
        assert!(stdout.contains("[2,]{name,role}:"));
        assert!(stdout.contains("Alice,admin"));
        assert!(stdout.contains("Bob,user"));
    }

    /// Test performance for small input
    #[test]
    fn test_stdin_performance_small() {
        use std::time::Instant;
        
        let input = r#"{"name": "Alice", "age": 30, "active": true}"#;
        
        let start = Instant::now();
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        let elapsed = start.elapsed();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should complete very quickly for small input
        assert!(elapsed.as_millis() < 100, "Small input should convert in under 100ms, took {:?}", elapsed);
    }

    /// Test stdin with verbose flag
    #[test]
    fn test_stdin_with_verbose() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin", "--verbose"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should contain conversion output
        assert!(stdout.contains("name:"));
        assert!(stdout.contains("Alice"));
        
        // With verbose, might have additional output, but should still work
        assert!(stdout.contains("age:"));
        assert!(stdout.contains("30"));
    }

    /// Test stdin help message
    #[test]
    fn test_stdin_help_message() {
        let (stdout, stderr) = run_toonconv_stdin("", &["--stdin", "--help"]).unwrap();
        
        // Should show help and exit successfully
        assert!(!stdout.is_empty(), "Should show help message");
        assert!(stdout.contains("toonconv") || stdout.contains("Convert JSON") || stdout.contains("usage"),
               "Should contain help information: {}", stdout);
    }

    /// Test multiline JSON from stdin
    #[test]
    fn test_stdin_multiline_json() {
        let input = r#"{
  "user": {
    "name": "Alice",
    "profile": {
      "age": 30,
      "location": "New York"
    }
  }
}"#;
        
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should handle multiline JSON correctly
        assert!(stdout.contains("user:"));
        assert!(stdout.contains("name:"));
        assert!(stdout.contains("Alice"));
        assert!(stdout.contains("profile:"));
        assert!(stdout.contains("age:"));
        assert!(stdout.contains("30"));
        assert!(stdout.contains("location:"));
        assert!(stdout.contains("New York"));
    }

    /// Test stdin with mixed data types
    #[test]
    fn test_stdin_mixed_data_types() {
        let input = r#"{
  "string": "hello",
  "number": 42,
  "float": 3.14,
  "boolean": true,
  "null": null,
  "array": [1, 2, 3],
  "object": {"key": "value"}
}"#;
        
        let (stdout, stderr) = run_toonconv_stdin(input, &["--stdin"]).unwrap();
        
        assert!(stderr.is_empty(), "Should not have errors: {}", stderr);
        assert!(!stdout.is_empty(), "Should have TOON output");
        
        // Should handle all data types correctly
        assert!(stdout.contains("string:"));
        assert!(stdout.contains("hello"));
        assert!(stdout.contains("number:"));
        assert!(stdout.contains("42"));
        assert!(stdout.contains("float:"));
        assert!(stdout.contains("3.14"));
        assert!(stdout.contains("boolean:"));
        assert!(stdout.contains("true"));
        assert!(stdout.contains("null:"));
        assert!(stdout.contains("null"));
        assert!(stdout.contains("array:"));
        assert!(stdout.contains("[3]:"));
        assert!(stdout.contains("1,2,3"));
        assert!(stdout.contains("object:"));
        assert!(stdout.contains("key:"));
        assert!(stdout.contains("value"));
    }
}