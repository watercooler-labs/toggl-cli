//! Stable-Rust release helper. Run through `./pkg/autodoc`.
use std::{error::Error, fs, process::Command};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
struct Options {
    version: String,
    commit: bool,
    check: bool,
    refresh: bool,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Options> {
    let mut version = None;
    let mut commit = false;
    let mut check = false;
    let mut refresh = false;
    for arg in args {
        match arg.as_str() {
            "--commit" => commit = true,
            "--check" => check = true,
            "--refresh" => refresh = true,
            _ if arg.starts_with('-') => return Err(format!("unknown flag: {arg}").into()),
            _ if version.is_some() => return Err("specify only one version or segment".into()),
            _ => version = Some(arg),
        }
    }
    if (check || refresh) && (commit || version.is_some() || (check && refresh)) {
        return Err("--check and --refresh cannot be combined with other options".into());
    }
    Ok(Options {
        version: version.unwrap_or_else(|| "patch".into()),
        commit,
        check,
        refresh,
    })
}

fn version_parts(version: &str) -> Result<[u64; 3]> {
    let parts = version
        .split('.')
        .map(|part| {
            if part.is_empty()
                || (part.len() > 1 && part.starts_with('0'))
                || !part.bytes().all(|b| b.is_ascii_digit())
            {
                return Err("expected a stable MAJOR.MINOR.PATCH version".into());
            }
            part.parse().map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;
    parts
        .try_into()
        .map_err(|_| "expected a stable MAJOR.MINOR.PATCH version".into())
}

fn next_version(current: &str, requested: &str) -> Result<String> {
    let old = version_parts(current)?;
    let mut new = old;
    match requested {
        "major" => new = [old[0].checked_add(1).ok_or("version overflow")?, 0, 0],
        "minor" => new = [old[0], old[1].checked_add(1).ok_or("version overflow")?, 0],
        "patch" => new[2] = old[2].checked_add(1).ok_or("version overflow")?,
        _ => new = version_parts(requested)?,
    }
    if new <= old {
        return Err("new version must be greater than the current version".into());
    }
    Ok(format!("{}.{}.{}", new[0], new[1], new[2]))
}

fn package_version(manifest: &str) -> Result<(usize, String)> {
    let mut in_package = false;
    for (index, line) in manifest.lines().enumerate() {
        if line.starts_with('[') {
            in_package = line.trim() == "[package]";
        }
        if in_package {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "version" {
                    let version = value.trim().trim_matches('"');
                    version_parts(version)?;
                    return Ok((index, version.into()));
                }
            }
        }
    }
    Err("package version not found in Cargo.toml".into())
}

fn checked_output(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "{command:?} failed ({}):\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn replace_help(readme: &str, marker: &str, help: &str) -> Result<String> {
    let mut lines = readme.lines();
    let mut result = String::new();
    let mut found = false;
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push('\n');
        // Exact matching prevents the general help block from replacing start help.
        if line == marker {
            if found {
                return Err(format!("duplicate README marker: {marker}").into());
            }
            found = true;
            for line in help.trim_end().lines() {
                result.push_str(line.trim_end());
                result.push('\n');
            }
            result.push('\n');
            let end = lines
                .by_ref()
                .find(|line| line.starts_with("```"))
                .ok_or("unterminated README help block")?;
            result.push_str(end);
            result.push('\n');
        }
    }
    if !found {
        return Err(format!("README marker not found: {marker}").into());
    }
    Ok(result)
}

fn generate_readme(readme: &str) -> Result<String> {
    let help =
        checked_output(Command::new("cargo").args(["run", "--locked", "--quiet", "--", "help"]))?;
    let start = checked_output(
        Command::new("cargo").args(["run", "--locked", "--quiet", "--", "help", "start"]),
    )?;
    let readme = replace_help(readme, "$ toggl help", &help)?;
    replace_help(&readme, "$ toggl help start", &start)
}

fn run() -> Result<()> {
    let options = parse_args(std::env::args().skip(1))?;
    let manifest = fs::read_to_string("Cargo.toml")?;
    let lock = fs::read_to_string("Cargo.lock")?;
    let readme = fs::read_to_string("README.md")?;
    if options.check || options.refresh {
        let updated = generate_readme(&readme)?;
        if options.refresh {
            fs::write("README.md", updated)?;
        } else if updated != readme {
            return Err("README help is stale; run ./pkg/autodoc --refresh".into());
        }
        return Ok(());
    }
    if !checked_output(Command::new("git").args(["status", "--porcelain"]))?.is_empty() {
        return Err("commit or stash your changes before preparing a release".into());
    }
    let (line_number, current) = package_version(&manifest)?;
    let version = next_version(&current, &options.version)?;
    println!("Bumping version from {current} to {version}");
    let new_manifest = manifest
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == line_number {
                format!("version = \"{version}\"")
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let prepare = || -> Result<()> {
        fs::write("Cargo.toml", new_manifest)?;
        // Only the root package version needs to change in the lockfile.
        checked_output(Command::new("cargo").arg("build"))?;
        fs::write("README.md", generate_readme(&readme)?)?;
        Ok(())
    };
    if let Err(error) = prepare() {
        fs::write("Cargo.toml", manifest)?;
        fs::write("Cargo.lock", lock)?;
        fs::write("README.md", readme)?;
        return Err(error);
    }
    if options.commit {
        checked_output(Command::new("git").args(["add", "Cargo.toml", "Cargo.lock", "README.md"]))?;
        checked_output(Command::new("git").args([
            "commit",
            "-m",
            &format!("🚢 Prepare release {version}"),
        ]))?;
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_flag_does_not_consume_the_version_argument() {
        for args in [
            vec!["minor"],
            vec!["minor", "--commit"],
            vec!["--commit", "minor"],
        ] {
            let options = parse_args(args.into_iter().map(String::from)).unwrap();
            assert_eq!(options.version, "minor");
        }
        assert_eq!(parse_args(Vec::new()).unwrap().version, "patch");
        assert!(parse_args(["--bogus".into()]).is_err());
        assert!(parse_args(["--check".into(), "minor".into()]).is_err());
    }

    #[test]
    fn bumps_and_validates_stable_versions() {
        assert_eq!(next_version("0.5.1", "minor").unwrap(), "0.6.0");
        assert_eq!(next_version("0.5.1", "major").unwrap(), "1.0.0");
        assert_eq!(next_version("0.5.1", "patch").unwrap(), "0.5.2");
        assert_eq!(next_version("0.5.1", "0.10.12").unwrap(), "0.10.12");
        for bad in ["1.2", "1.2.3.4", "01.2.3", "0.5.1", "0.4.0", "1.0.0-rc1"] {
            assert!(next_version("0.5.1", bad).is_err());
        }
        assert_eq!(
            package_version("[package]\nversion = \"0.10.12\"\n")
                .unwrap()
                .1,
            "0.10.12"
        );
    }

    #[test]
    fn help_replacement_is_exact_and_preserves_unicode() {
        let original = "$ toggl help\nold\n```\n$ toggl help start\nkeep\n```\n";
        let updated = replace_help(original, "$ toggl help", "new 🚢\n").unwrap();
        assert!(updated.contains("new 🚢\n\n```"));
        assert!(updated.contains("$ toggl help start\nkeep\n```"));
        assert!(replace_help("missing", "$ toggl help", "new").is_err());
    }

    #[test]
    fn failing_subprocesses_are_not_treated_as_success() {
        assert!(checked_output(Command::new("rustc").arg("--not-a-real-option")).is_err());
    }
}
