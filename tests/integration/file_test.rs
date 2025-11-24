//! Integration tests for file conversion workflow

#[cfg(test)]
mod file_conversion_tests {
    use tempfile::tempdir;
    use std::fs::{self, File};
    use std::io::Write;
    use std::process::Command;

    fn run_toonconv(args: &[&str]) -> Result<(String, String), String> {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", "toonconv", "--"]).args(args)
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        let output = cmd.output()
            .map_err(|e| format!("Failed to run toonconv: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        Ok((stdout, stderr))
    }

    #[test]
    fn test_directory_conversion_creates_output_files() {
        let input_dir = tempdir().unwrap();
        let nested = input_dir.path().join("sub");
        fs::create_dir_all(&nested).unwrap();

        // Create two JSON files
        let file1 = input_dir.path().join("a.json");
        let mut f1 = File::create(&file1).unwrap();
        write!(f1, "{{\"name\": \"Alice\"}}\n").unwrap();

        let file2 = nested.join("b.json");
        let mut f2 = File::create(&file2).unwrap();
        write!(f2, "{{\"name\": \"Bob\"}}\n").unwrap();

        let output_dir = tempdir().unwrap();

        let args = [input_dir.path().to_str().unwrap(), "--output", output_dir.path().to_str().unwrap(), "--recursive"];
        let (stdout, stderr) = run_toonconv(&args).unwrap();

        assert!(stderr.is_empty(), "No error expected: {}", stderr);
        assert!(stdout.contains("Found 2 JSON files") || stdout.contains("Found 1 JSON files") || stdout.contains("Converted"));

        // Check that output files were created
        let out1 = output_dir.path().join("a.toon");
        assert!(out1.exists(), "Expected output a.toon to exist");
        let out1_contents = fs::read_to_string(out1).unwrap();
        assert!(out1_contents.contains("name:"));
        assert!(out1_contents.contains("Alice"));

        let out2 = output_dir.path().join("sub/b.toon");
        assert!(out2.exists(), "Expected output sub/b.toon to exist");
        let out2_contents = fs::read_to_string(out2).unwrap();
        assert!(out2_contents.contains("name:"));
        assert!(out2_contents.contains("Bob"));
    }
}
