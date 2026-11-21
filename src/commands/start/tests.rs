use super::*;
use crate::{
    api::client::MockApiClient,
    error::ApiError,
    test_support::{self, CancelPicker},
};

fn client(reads: usize) -> MockApiClient {
    let mut client = MockApiClient::new();
    client
        .expect_get_user()
        .times(1)
        .returning(|| Ok(test_support::user()));
    client
        .expect_get_entities()
        .times(reads)
        .returning(|| Ok(test_support::entities()));
    client
}

#[tokio::test]
async fn invalid_project_or_task_never_stops_the_running_timer() {
    for (project, task) in [
        (Some("missing"), None),
        (Some("alpha"), Some("missing")),
        (None, Some("Development")),
    ] {
        let result = StartCommand::execute_with_config(
            client(1),
            Box::new(CancelPicker),
            None,
            project.map(String::from),
            None,
            false,
            false,
            task.map(String::from),
            |_| Ok(TimeEntry::default()),
        )
        .await;
        assert!(result.is_err());
        // MockApiClient has no write expectations: any stop/create would panic.
    }
}

#[tokio::test]
async fn bad_configuration_and_cancelled_selection_never_write() {
    let result = StartCommand::execute_with_config(
        client(1),
        Box::new(CancelPicker),
        None,
        None,
        None,
        false,
        false,
        None,
        |_| Err(Box::new(std::io::Error::other("bad config"))),
    )
    .await;
    assert!(result.is_err());
    let result = StartCommand::execute_with_config(
        client(1),
        Box::new(CancelPicker),
        None,
        None,
        None,
        false,
        true,
        None,
        |_| Ok(TimeEntry::default()),
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn project_overrides_keep_only_compatible_config_tasks() {
    for (project, task_flag, expected_task) in [
        ("alpha", None, Some(11)),
        ("beta", None, None),
        ("beta", Some("Development"), Some(22)),
    ] {
        let expected_project = if project == "alpha" { 1 } else { 2 };
        let mut client = client(2);
        client
            .expect_create_time_entry()
            .times(1)
            .withf(move |entry| {
                entry.project.as_ref().map(|p| p.id) == Some(expected_project)
                    && entry.task.as_ref().map(|t| t.id) == expected_task
                    && entry.description == "CLI description"
                    && entry.tags == ["cli-tag"]
            })
            .returning(|_| Ok(42));
        StartCommand::execute_with_config(
            client,
            Box::new(CancelPicker),
            Some("CLI description".into()),
            Some(project.into()),
            Some(vec!["cli-tag".into()]),
            false,
            false,
            task_flag.map(String::from),
            |entities| {
                Ok(TimeEntry {
                    project: entities.projects.get(&1).cloned(),
                    task: entities.tasks.get(&11).cloned(),
                    description: "config description".into(),
                    tags: vec!["config-tag".into()],
                    ..TimeEntry::default()
                })
            },
        )
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn stop_failure_does_not_create_a_second_timer() {
    let mut client = MockApiClient::new();
    client
        .expect_get_user()
        .returning(|| Ok(test_support::user()));
    client.expect_get_entities().times(2).returning(|| {
        let mut entities = test_support::entities();
        entities.time_entries.push(TimeEntry {
            id: 10,
            ..TimeEntry::default()
        });
        Ok(entities)
    });
    client
        .expect_update_time_entry()
        .times(1)
        .returning(|_| Err(Box::new(ApiError::Network)));
    assert!(StartCommand::execute_with_config(
        client,
        Box::new(CancelPicker),
        None,
        Some("alpha".into()),
        None,
        false,
        false,
        None,
        |_| Ok(TimeEntry::default()),
    )
    .await
    .is_err());
}
