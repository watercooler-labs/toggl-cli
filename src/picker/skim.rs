use crate::error;
use crate::models;
use crate::picker;
use error::PickerError;
use models::ResultWithDefaultError;
use picker::{ItemPicker, PickableItem};
use skim::prelude::*;

pub struct SkimPicker;

fn get_skim_configuration(items: Vec<PickableItem>) -> (SkimOptions<'static>, SkimItemReceiver) {
    let options = SkimOptionsBuilder::default()
        // Set viewport to take entire screen
        .height(Some("100%"))
        // Disable multiselect
        .multi(false)
        .build()
        .unwrap();

    let (sender, source): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in items {
        // Send items to Skim receiver
        let _ = sender.send(Arc::new(item));
    }
    // Complete sender transaction to signal no new items will be added after this
    drop(sender);
    (options, source)
}

impl SkimItem for PickableItem {
    fn text(&self) -> Cow<str> {
        Cow::from(self.formatted.as_str())
    }

    fn output(&self) -> Cow<str> {
        Cow::from(self.id.to_string())
    }
}

impl ItemPicker for SkimPicker {
    fn pick(&self, items: Vec<PickableItem>) -> ResultWithDefaultError<i64> {
        let (options, source) = get_skim_configuration(items);
        let output = Skim::run_with(&options, Some(source));

        match output {
            None => Err(Box::new(PickerError::Cancelled)),
            Some(item) => {
                if item.is_abort {
                    Err(Box::new(PickerError::Cancelled))
                } else {
                    let selectable_items = item
                        .selected_items
                        .iter()
                        .map(|selected_items| selected_items.output().parse::<i64>().unwrap())
                        .collect::<Vec<i64>>();

                    match selectable_items.first() {
                        None => Err(Box::new(PickerError::Generic)),
                        Some(id) => Ok(*id),
                    }
                }
            }
        }
    }
}
