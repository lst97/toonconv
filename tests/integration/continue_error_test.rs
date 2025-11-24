//! Integration tests for continue-on-error flag

#[cfg(test)]
mod continue_on_error_tests {
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
    fn test_abort_on_error_default() {
        let input_dir = tempdir().unwrap();
        let file_bad = input_dir.path().join("bad.json");
        let mut fb = File::create(&file_bad).unwrap();
        write!(fb, "{{ name: invalid }}").unwrap();

        let file_good = input_dir.path().join("good.json");
        let mut fg = File::create(&file_good).unwrap();
        write!(fg, "{{\"name\": \"OK\"}}\n").unwrap();

        let output_dir = tempdir().unwrap();
        let args = [input_dir.path().to_str().unwrap(), "--output", output_dir.path().to_str().unwrap(), "--recursive"];
        let (stdout, stderr) = run_toonconv(&args).unwrap();

        // Without --continue-on-error, abort on first error: good.json should not be converted
        assert!(!stderr.is_empty(), "Should report error: {}", stderr);
        let out_good = output_dir.path().join("good.toon");
        assert!(!out_good.exists());
    }

    #[test]
    fn test_continue_on_error_flag() {
        let input_dir = tempdir().unwrap();
        let file_bad = input_dir.path().join("bad.json");
        let mut fb = File::create(&file_bad).unwrap();
        write!(fb, "{{ name: invalid }}").unwrap();

        let file_good = input_dir.path().join("good.json");
        let mut fg = File::create(&file_good).unwrap();
        write!(fg, "{{\"name\": \"OK\"}}\n").unwrap();

        let output_dir = tempdir().unwrap();
        let args = [input_dir.path().to_str().unwrap(), "--output", output_dir.path().to_str().unwrap(), "--recursive", "--continue-on-error"];
        let (stdout, stderr) = run_toonconv(&args).unwrap();

        // With --continue-on-error, the good file should be converted despite error
        assert!(!stderr.is_empty(), "Should report error: {}", stderr);
        let out_good = output_dir.path().join("good.toon");
        assert!(out_good.exists());
    }
}
