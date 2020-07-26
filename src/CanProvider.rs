use std::collections::HashMap;

pub type CanData = HashMap<&'static str, String>;

pub trait CanProvider: std::marker::Sync {
    fn can_data(&mut self) -> CanData;
}
