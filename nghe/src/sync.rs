use loole::{Receiver, Sender};

pub fn channel<T>(size: Option<usize>) -> (Sender<T>, Receiver<T>) {
    if let Some(size) = size { loole::bounded(size) } else { loole::unbounded() }
}
