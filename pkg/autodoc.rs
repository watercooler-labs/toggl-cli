#!/usr/bin/env cargo +nightly -Zscript

// Returns current version of the package
// reads the Cargo.toml file and returns the version and line number
fn get_pkg_version() -> (String, usize) {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let mut line_number = 0;
    for line in cargo_toml.lines() {
        line_number += 1;
        if line.starts_with("version") {
            let version = line
                .split("=")
                .nth(1)
                .expect("Invariant state")
                .trim()
                .trim_matches('"')
                .to_string();
            return (version, line_number);
        }
    }
    return ("".to_string(), 0);
}

fn get_next_version(current_version: &str, segment_or_version: Option<&str>) -> String {
    let mut version = current_version
        .split(".")
        .map(|s| s.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();

    match segment_or_version.unwrap_or("patch") {
        "major" => {
            version[0] += 1;
            version[1] = 0;
            version[2] = 0;
        }
        "minor" => {
            version[1] += 1;
            version[2] = 0;
        }
        "patch" => version[2] += 1,
        _ => return segment_or_version.expect("Invariant state").to_string(),
    }

    version
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(".")
}

fn write_pkg_version(new_version: &str, line_number: usize) {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let mut curr_line_number = 0;
    let new_cargo_toml = cargo_toml
        .lines()
        .map(|line| {
            curr_line_number += 1;
            if curr_line_number == line_number {
                format!("version = \"{new_version}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    // Add final newline
    let new_cargo_toml = format!("{}\n", new_cargo_toml);
    // Write new Cargo.toml
    std::fs::write("Cargo.toml", new_cargo_toml).unwrap();
}

// Updates the README.md file matches the line prefix
// and replaces the contents from next line to the line containing "```"
// with the output of the Command passed
fn update_readme(line_prefix: &str, command: &mut std::process::Command) {
    let readme = std::fs::read_to_string("README.md").unwrap();

    let mut new_readme = Vec::new();
    let output = command.output().expect("Failed to execute command");
    // Trim output to remove trailing space on each line
    let output_lines = output
        .stdout
        .iter()
        .map(|b| *b as char)
        .collect::<String>()
        .split("\n")
        .map(|s| s.to_string().trim_end().to_string())
        .collect::<Vec<String>>()
        .join("\n");

    // Consume iterator until line_prefix is found
    // Then merge output lines and skip iterator until "```" is found
    // append the output lines to the new_readme vector
    let mut lines = readme.lines();
    while let Some(line) = lines.next() {
        if line.starts_with(line_prefix) {
            new_readme.push(line.to_string());
            new_readme.push(output_lines.clone());
            while let Some(line) = lines.next() {
                if line.starts_with("```") {
                    new_readme.push(line.to_string());
                    break;
                }
            }
        } else {
            new_readme.push(line.to_string());
        }
    }
    // Add final newline
    new_readme.push("".to_string());

    // Write new README.md
    std::fs::write("README.md", new_readme.join("\n")).unwrap();
}

fn main() {
    let (current_version, line_number) = get_pkg_version();

    let args = std::env::args();
    let arg = args.skip(1).next();
    let segment_or_version = arg.as_deref();

    // Bump version
    let new_version = get_next_version(&current_version, segment_or_version);
    println!("Bumping version from {current_version} to {new_version}");

    // Update Cargo.toml at line number
    write_pkg_version(&new_version, line_number);

    // Invoke cargo to build the package
    println!("Building package");
    std::process::Command::new("cargo")
        .arg("build")
        .output()
        .expect("Failed to build package");

    // Update README.md
    println!("Updating README.md");
    update_readme(
        "$ toggl help",
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("help"),
    );
    update_readme(
        "$ toggl help start",
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("help")
            .arg("start"),
    );
}
