use crate::error;
use crate::models;
use crate::picker;
use error::PickerError;
use models::ResultWithDefaultError;
use picker::{ItemPicker, PickableItem};
use skim::prelude::*;

pub struct SkimPicker;

impl SkimItem for PickerItem {
    
    fn text(&self) -> Cow<str> {
        Cow::from(self.formatted());
    }

    fn output(&self) -> Cow<str> {
        Cow::from(self.id().to_string())
    }
}

impl ItemPicker for SkimPicker {

    fn pick<T: PickableItem>(&self, items: Vec<T>) -> ResultWithDefaultError<T> {
        
        let (options, source) = get_skim_configuration(time_entries.clone());
        let output = Skim::run_with(&options, Some(source));
        if output.is_abort {
            return Err(Box::new(PickerError::Cancelled));
        } 
        
        return output.selected_items
            .map(|selected_items| {
                selected_items
                    .first()
                    .map(|item| item.output().parse::<i64>().unwrap())
            })
            .and_then(|selected_id| match selected_id {
                Some(id) => Ok(
                    items
                        .iter()
                        .find(|item| item.id() == id)
                        .cloned()
                ),
                _ => Err(Box::new(PickerError::Generic)),
            })
    }

    fn get_skim_configuration<T : SkimItem>(items: Vec<T>) -> (SkimOptions<'static>, SkimItemReceiver) {
        let options = SkimOptionsBuilder::default()
            // Set viewport to take entire screen
            .height(Some("100%"))
            // Disable multiselect
            .multi(false)
            .build()
            .unwrap();

        let (sender, source): (SkimItemSender, SkimItemReceiver) = unbounded();
        for item in items {
            // Send time-entries to Skim receiver
            let _ = sender.send(Arc::new(item.clone()));
        }
        // Complete sender transaction to signal no new items will be added after this
        drop(sender);
        (options, source)
    }
}