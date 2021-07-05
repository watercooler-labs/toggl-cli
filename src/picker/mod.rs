pub trait PickableItem : Clone {
    fn id(&self) -> i64;
    fn formatted(&self) -> String;
}

pub trait ItemPicker {
    fn pick<T: PickableItem>(&self, items: Vec<T>) -> ResultWithDefaultError<T>;
}

