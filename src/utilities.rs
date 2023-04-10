use std::{
    io::{self, Write},
    path::Path,
};

use colored::Colorize;

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

pub fn open_path_in_editor(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let mut child = std::process::Command::new(editor)
        .arg(path)
        .spawn()
        .expect("Failed to spawn editor");
    child.wait().expect("Failed to wait for editor");
    Ok(())
}

fn print_without_buffer(text: &str) {
    print!("{}", text);
    io::stdout().flush().unwrap();
}