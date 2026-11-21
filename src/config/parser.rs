use std::{collections::BTreeMap, path::Path};

use serde::Deserialize;

use crate::{models::ResultWithDefaultError, utilities};

use super::model::TrackConfig;

fn read_config(path: &Path) -> ResultWithDefaultError<String> {
    std::fs::read_to_string(path).map_err(|err| -> Box<dyn std::error::Error + Send> {
        Box::new(std::io::Error::other(format!(
            "cannot read config {}: {err}",
            path.display()
        )))
    })
}

pub fn get_config_from_file<P: AsRef<Path>>(path: P) -> ResultWithDefaultError<TrackConfig> {
    let path = path.as_ref();
    let contents = read_config(path)?;
    toml::from_str(&contents).map_err(|err| -> Box<dyn std::error::Error + Send> {
        Box::new(std::io::Error::other(format!(
            "{}\ncannot parse config {}: {err}",
            crate::error::ConfigError::Parse,
            path.display()
        )))
    })
}

/// Read only the credential selector. Deserializing BranchConfig would also
/// execute shell templates in unrelated descriptions, projects and tags.
pub fn get_profile_from_file<P: AsRef<Path>>(path: P) -> ResultWithDefaultError<Option<String>> {
    let contents = read_config(path.as_ref())?;
    let dir = std::env::current_dir()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    profile_for_branch(
        &contents,
        utilities::get_git_branch_for_dir(&dir).as_deref(),
    )
}

fn profile_for_branch(
    contents: &str,
    branch: Option<&str>,
) -> ResultWithDefaultError<Option<String>> {
    #[derive(Deserialize)]
    struct ProfileConfig {
        profile: Option<String>,
    }

    // Match TrackConfig's TOML table ordering and whole-block selection: a
    // matching branch without a profile does not inherit the default's profile.
    let configs: BTreeMap<String, ProfileConfig> = toml::from_str(contents)
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    if let Some(branch) = branch {
        for (pattern, config) in &configs {
            if pattern == "*" {
                continue;
            }
            let re = regex::Regex::new(pattern)
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
            if re.is_match(branch) {
                return Ok(config.profile.clone());
            }
        }
    }
    Ok(configs.get("*").and_then(|config| config.profile.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_profiles_without_expanding_templates() {
        let config = "['*']\nprofile = 'personal'\ndescription = '{{unknown}}'\n\
                      ['^work/']\nprofile = 'work'\n['^main$']\nproject = 'project'\n";
        assert_eq!(
            profile_for_branch(config, None).unwrap().as_deref(),
            Some("personal")
        );
        assert_eq!(
            profile_for_branch(config, Some("work/fix"))
                .unwrap()
                .as_deref(),
            Some("work")
        );
        assert_eq!(profile_for_branch(config, Some("main")).unwrap(), None);
    }

    #[test]
    fn invalid_profile_config_and_patterns_return_errors() {
        assert!(profile_for_branch("invalid = [", None).is_err());
        assert!(profile_for_branch("['[']\nprofile = 'work'", Some("main")).is_err());
    }

    #[test]
    fn missing_and_malformed_files_return_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(get_config_from_file(&path).is_err());
        std::fs::write(&path, "invalid = [").unwrap();
        assert!(get_config_from_file(&path).is_err());
    }
}
