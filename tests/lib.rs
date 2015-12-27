use std::io::Result;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

fn get_sample_path(filename: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/input");
    path.push(filename);
    path.to_string_lossy().to_string()
}

fn get_exe_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/debug/cara");
    path
}

struct RunOutput {
    stdout: String,
    stderr: String,
    status: ExitStatus,
}

fn run_test(sample_file: &str, resign_rule: &str, draw_rule: &str, verbose: bool)
    -> Result<RunOutput>
{
    let mut exe = Command::new(get_exe_path());
    let base = exe.arg("test")
        .arg(get_sample_path(sample_file))
        .arg(resign_rule)
        .arg(draw_rule);

    let command = if verbose {
        base.arg("--verbose")
    } else {
        base
    };

    let result = command.output();

    match result {
        Ok(output) => Ok(RunOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status }),
        Err(error) => Err(error),
    }

}

#[test]
fn no_args() {
    let output = Command::new(get_exe_path()).output().unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let err_str = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output_str, "");
    assert!(err_str.contains("USAGE:"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_command_cant_open_file() {
    let output = run_test("missing.pgn", "none", "none", false).unwrap();

    assert_eq!(output.stdout, "".to_string());
    assert_eq!(output.stderr, "error: Can't open file\n".to_string());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_command_no_rules() {
    let output = run_test("resign.pgn", "none", "none", true).unwrap();

    assert_eq!(
        output.stdout,
        concat!(
            "game, actual_length, actual_time, actual_score, ",
            "rule_applied, adjudicated_length, adjudicated_time, adjudicated_score\n",
            "1, 73, 16590, 0.5, -, 73, 16590, 0.5\n",
            "2, 159, 22520, 1, -, 159, 22520, 1\n",
            "3, 160, 22432, 0, -, 160, 22432, 0\n",
            "4, 141, 22478, 0.5, -, 141, 22478, 0.5\n",
            "5, 512, 33966, 0.5, -, 512, 33966, 0.5\n",
            "\n",
            "Games: 5\n",
            "Adjudicated: 0 (0 wrong)\n",
            "  Resign: 0 (0 wrong)\n",
            "  Draw: 0 (0 wrong)\n",
            "\n",
            "Total Time: 0:01:57.986\n",
            "After Adjudication: 0:01:57.986\n",
            "Time saved: 0:00:00.000 (0.00%)\n",
            "  Resign: 0:00:00.000 (0.00%)\n",
            "  Draw: 0:00:00.000 (0.00%)\n",
            "Note: 'Time saved' excludes incorrectly adjudicated games\n",
            "\n",
            "Mean Squared Error: 0.000000\n",
            "  Resign: 0.000000\n",
            "  Draw: 0.000000\n",
            "Root MSE: 0.000\n")
    );
}


#[test]
fn test_command_resign_rule() {
    let output = run_test("resign.pgn", "250/3", "none", true).unwrap();

    assert_eq!(
        output.stdout,
        concat!(
            "game, actual_length, actual_time, actual_score, ",
            "rule_applied, adjudicated_length, adjudicated_time, adjudicated_score\n",
            "1, 73, 16590, 0.5, -, 73, 16590, 0.5\n",
            "2, 159, 22520, 1, R, 118, 21180, 1\n",
            "3, 160, 22432, 0, R, 117, 20698, 0\n",
            "4, 141, 22478, 0.5, R, 123, 21888, 0\n",
            "5, 512, 33966, 0.5, R, 110, 19640, 1\n",
            "\n",
            "Games: 5\n",
            "Adjudicated: 4 (2 wrong)\n",
            "  Resign: 4 (2 wrong)\n",
            "  Draw: 0 (0 wrong)\n",
            "\n",
            "Total Time: 0:01:57.986\n",
            "After Adjudication: 0:01:39.996\n",
            "Time saved: 0:00:03.074 (2.61%)\n",
            "  Resign: 0:00:03.074 (2.61%)\n",
            "  Draw: 0:00:00.000 (0.00%)\n",
            "Note: 'Time saved' excludes incorrectly adjudicated games\n",
            "\n",
            "Mean Squared Error: 0.100000\n",
            "  Resign: 0.100000\n",
            "  Draw: 0.000000\n",
            "Root MSE: 0.316\n")
    );
}


#[test]
fn test_command_draw_rule() {
    let output = run_test("draw.pgn", "none", "34:30/8", true).unwrap();

    assert_eq!(
        output.stdout,
        concat!(
            "game, actual_length, actual_time, actual_score, ",
            "rule_applied, adjudicated_length, adjudicated_time, adjudicated_score\n",
            "1, 55, 11406, 0.5, -, 55, 11406, 0.5\n",
            "2, 73, 16862, 0.5, -, 73, 16862, 0.5\n",
            "3, 73, 16590, 0.5, D, 68, 16220, 0.5\n",
            "4, 151, 22138, 1, D, 68, 15652, 0.5\n",
            "5, 190, 23480, 0, D, 94, 18851, 0.5\n",
            "\n",
            "Games: 5\n",
            "Adjudicated: 3 (2 wrong)\n",
            "  Resign: 0 (0 wrong)\n",
            "  Draw: 3 (2 wrong)\n",
            "\n",
            "Total Time: 0:01:30.476\n",
            "After Adjudication: 0:01:18.991\n",
            "Time saved: 0:00:00.370 (0.41%)\n",
            "  Resign: 0:00:00.000 (0.00%)\n",
            "  Draw: 0:00:00.370 (0.41%)\n",
            "Note: 'Time saved' excludes incorrectly adjudicated games\n",
            "\n",
            "Mean Squared Error: 0.100000\n",
            "  Resign: 0.000000\n",
            "  Draw: 0.100000\n",
            "Root MSE: 0.316\n")
    );
}
