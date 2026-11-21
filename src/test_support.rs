use crate::{
    models::*,
    picker::{ItemPicker, PickableItem, PickableItemKey},
};

pub fn user() -> User {
    User {
        api_token: "test-token".into(),
        email: "test@example.com".into(),
        fullname: None,
        timezone: "UTC".into(),
        default_workspace_id: 1,
    }
}

pub fn entities() -> Entities {
    let projects = [(1, "alpha"), (2, "beta")]
        .into_iter()
        .map(|(id, name)| {
            (
                id,
                Project {
                    id,
                    name: name.into(),
                    workspace_id: 1,
                    ..Project::default()
                },
            )
        })
        .collect::<std::collections::HashMap<_, _>>();
    let tasks = projects
        .values()
        .map(|project| {
            let id = project.id * 11;
            (
                id,
                Task {
                    id,
                    name: "Development".into(),
                    workspace_id: 1,
                    project: project.clone(),
                },
            )
        })
        .collect();
    Entities {
        projects,
        tasks,
        time_entries: Vec::new(),
        clients: Default::default(),
        workspaces: vec![Workspace {
            id: 1,
            name: "workspace".into(),
            admin: true,
        }],
        tags: Vec::new(),
    }
}

pub struct CancelPicker;

impl ItemPicker for CancelPicker {
    fn pick(&self, _items: Vec<PickableItem>) -> ResultWithDefaultError<PickableItemKey> {
        Err(Box::new(crate::error::PickerError::Cancelled))
    }
}
