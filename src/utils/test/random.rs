use rand::seq::IteratorRandom;

pub fn gen_bool_mask(len: usize, n_true: usize) -> Vec<bool> {
    let true_idx = (0..len).choose_multiple(&mut rand::thread_rng(), n_true);
    (0..len).map(|i| true_idx.contains(&i)).collect()
}
