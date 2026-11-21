use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_toggl"))
}

#[test]
fn profile_examples_parse_without_accessing_credentials() {
    for subcommand in ["auth", "logout"] {
        let output = binary()
            .args([subcommand, "--profile", "work", "--help"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[cfg(unix)]
mod isolated_config {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use std::path::PathBuf;

    struct Sandbox {
        _temp: tempfile::TempDir,
        root: PathBuf,
        config: PathBuf,
    }

    impl Sandbox {
        fn new(contents: &str) -> Self {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            #[cfg(target_os = "macos")]
            let config_root =
                root.join("Library/Application Support/studio.watercooler.labs.toggl-cli");
            #[cfg(not(target_os = "macos"))]
            let config_root = root.join(".config/toggl-cli");
            let config = config_root.join(format!(
                "{}.toml",
                general_purpose::STANDARD.encode(root.to_str().unwrap())
            ));
            std::fs::create_dir_all(config.parent().unwrap()).unwrap();
            std::fs::write(&config, contents).unwrap();
            Self {
                _temp: temp,
                root,
                config,
            }
        }

        fn command(&self) -> Command {
            let mut command = binary();
            command
                .current_dir(&self.root)
                .env("HOME", &self.root)
                .env("XDG_CONFIG_HOME", self.root.join(".config"))
                .env_remove("TOGGL_PROFILE")
                .env("TOGGL_API_TOKEN", "dummy-test-token");
            command
        }
    }

    #[test]
    fn completions_do_not_read_broken_configuration() {
        let sandbox = Sandbox::new("invalid = [");
        for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
            let output = sandbox
                .command()
                .args(["completions", "--shell", shell])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty());
            assert!(!output.stdout.is_empty());
        }
    }

    #[test]
    fn config_path_and_delete_can_recover_from_malformed_toml() {
        let sandbox = Sandbox::new("invalid = [");
        let output = sandbox
            .command()
            .args(["config", "--path"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            sandbox.config.to_str().unwrap()
        );
        let output = sandbox
            .command()
            .args(["config", "--delete"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!sandbox.config.exists());
    }

    #[test]
    fn malformed_config_is_an_error_not_a_panic() {
        let sandbox = Sandbox::new("invalid = [");
        let output = sandbox.command().args(["config"]).output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }

    #[test]
    fn profile_resolution_never_expands_unrelated_templates() {
        let sandbox = Sandbox::new("['*']\ndescription = '{{$ touch TEMPLATE_RAN}}'\n");
        // An invalid API path is rejected locally, with no request or keyring access.
        let output = sandbox
            .command()
            .args(["api", "invalid-path"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid API path"));
        assert!(output.stdout.is_empty());
        assert!(!sandbox.root.join("TEMPLATE_RAN").exists());
    }

    #[test]
    fn network_errors_exit_nonzero_without_emitting_json() {
        let sandbox = Sandbox::new("");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        for args in [
            vec!["list", "--json"],
            vec!["list", "--json", "--since", "2026-11-20"],
            vec!["api", "/me"],
        ] {
            let output = sandbox
                .command()
                .args(["--proxy", &proxy])
                .args(args)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(
                output.stdout.is_empty(),
                "{}",
                String::from_utf8_lossy(&output.stdout)
            );
            assert!(!output.stderr.is_empty());
        }
    }

    #[test]
    fn invalid_templates_do_not_pollute_stdout() {
        let sandbox = Sandbox::new("['*']\ndescription = '{{unknown_macro}}'\n");
        let output = sandbox.command().args(["config"]).output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Unrecognized macro"));
    }
}
