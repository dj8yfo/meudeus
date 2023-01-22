use rand::{distributions::Alphanumeric, Rng}; // 0.8

pub fn rand_suffix() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}