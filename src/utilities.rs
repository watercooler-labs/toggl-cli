use std::path::Path;

pub fn remove_trailing_newline(value: String) -> String {
    value.trim_end().to_string()
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
