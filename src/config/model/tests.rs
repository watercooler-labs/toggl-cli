use super::*;

#[test]
fn config_tasks_are_scoped_to_the_configured_project() {
    for (project, expected) in [
        (Some("alpha"), Some(11)),
        (Some("beta"), Some(22)),
        (None, None),
    ] {
        let config = TrackConfig {
            default: BranchConfig {
                project: project.map(String::from),
                task: Some("Development".into()),
                ..Default::default()
            },
            configs: Vec::new(),
        };
        let entry = config
            .get_default_entry(crate::test_support::entities())
            .unwrap();
        assert_eq!(entry.task.map(|task| task.id), expected);
    }
}

#[test]
fn invalid_branch_regex_returns_an_error() {
    let config = TrackConfig {
        default: BranchConfig::default(),
        configs: vec![("[".into(), BranchConfig::default())],
    };
    assert!(config.get_branch_config(Some("main")).is_err());
}
