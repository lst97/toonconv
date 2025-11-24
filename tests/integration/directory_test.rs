//! Integration tests for batch directory conversion

#[cfg(test)]
mod batch_tests {
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
    fn test_batch_directory_conversion() {
        let input_dir = tempdir().unwrap();
        let nested = input_dir.path().join("sub");
        fs::create_dir_all(&nested).unwrap();

        // Create files
        let file1 = input_dir.path().join("a.json");
        let mut f1 = File::create(&file1).unwrap();
        write!(f1, "{{\"name\": \"Alice\"}}\n").unwrap();

        let file2 = nested.join("b.txt");
        let mut f2 = File::create(&file2).unwrap();
        write!(f2, "not json").unwrap();

        let output_dir = tempdir().unwrap();
        let args = [input_dir.path().to_str().unwrap(), "--output", output_dir.path().to_str().unwrap(), "--recursive"];
        let (stdout, stderr) = run_toonconv(&args).unwrap();

        assert!(stderr.is_empty(), "No error expected: {}", stderr);
        assert!(stdout.contains("Found"), "Should report found files: {}", stdout);

        // JSON file should create output file
        let out1 = output_dir.path().join("a.toon");
        assert!(out1.exists());

        // Non-JSON file should be ignored (no .toon created)
        let out2 = output_dir.path().join("sub/b.toon");
        assert!(!out2.exists());
    }
}
