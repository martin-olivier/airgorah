use gtk4::prelude::*;
use gtk4::*;

#[macro_export]
macro_rules! list_store_get {
    ($storage:expr,$iter:expr,$pos:expr,$typ:ty) => {
        $storage.get_value($iter, $pos).get::<$typ>().unwrap()
    };
}

pub fn list_store_find(storage: &ListStore, pos: i32, to_match: &str) -> Option<TreeIter> {
    let mut iter = storage.iter_first();

    while let Some(it) = iter {
        let value = list_store_get!(storage, &it, pos, String);

        if value == to_match {
            return Some(it);
        }

        iter = match storage.iter_next(&it) {
            true => Some(it),
            false => None,
        }
    }

    None
}
