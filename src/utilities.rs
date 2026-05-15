use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;
use directories::BaseDirs;

use crate::{constants, models::ResultWithDefaultError};

pub fn remove_trailing_newline(value: String) -> String {
    value.trim_end().to_string()
}

pub fn read_from_stdin(text: &str) -> String {
    print_without_buffer(text);
    let mut result = String::new();
    io::stdin()
        .read_line(&mut result)
        .expect("Failed to read line");
    remove_trailing_newline(result)
}

pub fn simplify_config_path_for_display(dir: &Path) -> String {
    if !std::io::stdout().is_terminal() {
        return dir.display().to_string();
    }
    let base_dirs = BaseDirs::new().unwrap();
    let local_config_base_path = base_dirs.config_local_dir().to_str().unwrap();
    let mut display_config_path = dir.to_str().unwrap().to_string();
    display_config_path.replace_range(..local_config_base_path.len(), constants::SIMPLE_HOME_PATH);

    display_config_path
}

pub fn read_from_stdin_with_constraints(text: &str, valid_values: &[String]) -> String {
    loop {
        let result = read_from_stdin(text);
        if valid_values.contains(&result) {
            return result;
        } else {
            let error_message = format!(
                "Invalid value \"{}\". Valid values are: {}\n",
                result,
                valid_values.join(", ")
            )
            .red();
            print_without_buffer(&error_message);
        }
    }
}

pub fn open_path_in_editor<P>(path: P) -> ResultWithDefaultError<()>
where
    P: AsRef<Path>,
{
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vim".to_string());
    let mut child = std::process::Command::new(&editor)
        .arg(path.as_ref())
        .spawn()
        .map_err(|e| -> Box<dyn std::error::Error + Send> {
            Box::new(io::Error::other(format!(
                "Failed to spawn editor \"{editor}\": {e}"
            )))
        })?;
    let status = child
        .wait()
        .map_err(|e| -> Box<dyn std::error::Error + Send> {
            Box::new(io::Error::other(format!("Editor wait failed: {e}")))
        })?;
    if !status.success() {
        return Err(Box::new(io::Error::other(format!(
            "Editor \"{editor}\" exited with status {status}"
        ))));
    }
    Ok(())
}

pub fn get_git_branch_for_dir(dir: &PathBuf) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8(output.stdout).ok()?;
    Some(branch.trim().to_string())
}

#[cfg(unix)]
pub fn get_shell_cmd(command: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c");
    cmd.arg(command);
    cmd
}

#[cfg(windows)]
pub fn get_shell_cmd(command: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/C");
    cmd.arg(command);
    cmd
}

fn print_without_buffer(text: &str) {
    print!("{text}");
    io::stdout().flush().unwrap();
}
