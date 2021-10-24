use std::fmt::Debug;

pub fn ok_or_log<T, E: Debug>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(x) => Some(x),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}
