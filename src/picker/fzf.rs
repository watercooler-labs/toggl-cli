use crate::error;
use crate::models;
use crate::picker;
use crate::utilities;
use error::PickerError;
use models::ResultWithDefaultError;
use picker::{ItemPicker, PickableItem};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

use super::PickableItemKey;

pub struct FzfPicker;

fn format_as_fzf_input(items: &[PickableItem]) -> String {
    items
        .iter()
        .map(|item| item.formatted.clone())
        .fold("".to_string(), |acc, item| acc + item.as_str() + "\n")
}

fn create_element_hash_map(items: &[PickableItem]) -> HashMap<String, PickableItemKey> {
    items
        .iter()
        .map(|item| (item.formatted.clone(), item.key.clone()))
        .collect()
}

impl ItemPicker for FzfPicker {
    fn pick(&self, items: Vec<PickableItem>) -> ResultWithDefaultError<PickableItemKey> {
        let mut command = Command::new("fzf");
        command
            .arg("-n2..")
            .arg("--ansi")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        match command.spawn() {
            Ok(mut child) => {
                let fzf_input = format_as_fzf_input(&items);
                let possible_elements = create_element_hash_map(&items);

                writeln!(child.stdin.as_mut().unwrap(), "{}", fzf_input)?;

                match child.wait_with_output() {
                    Err(_) => Err(Box::new(PickerError::Generic)),
                    Ok(output) => match output.status.code() {
                        Some(0) => {
                            let user_selected_string = String::from_utf8(output.stdout)?;
                            let selected_item_index =
                                utilities::remove_trailing_newline(user_selected_string);
                            let selected_item =
                                possible_elements.get(&selected_item_index).unwrap();
                            Ok(selected_item.clone())
                        }
                        // This is copied from zoxide's fzf handler.
                        // https://github.com/rohankumardubey/zoxide/blob/main/src/util.rs
                        Some(128..=254) | None => Err(Box::new(PickerError::Cancelled)),
                        _ => Err(Box::new(PickerError::Generic)),
                    },
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(Box::new(PickerError::FzfNotInstalled))
            }
            Err(_) => Err(Box::new(PickerError::Generic)),
        }
    }
}
