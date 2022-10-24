use gtk4::prelude::*;
use gtk4::*;

pub fn list_store_find(storage: &ListStore, pos: i32, to_match: &str) -> Option<TreeIter> {
    let mut iter = storage.iter_first();

    while let Some(it) = iter {
        let value = storage.get_value(&it, pos);
        let value_as_str = value.get::<&str>().unwrap();

        if value_as_str == to_match {
            return Some(it);
        }

        iter = match storage.iter_next(&it) {
            true => Some(it),
            false => None,
        }
    }

    None
}
