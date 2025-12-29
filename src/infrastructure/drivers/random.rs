use esp_hal::rng::Rng;

pub(crate) fn get_seed() -> u64 {
    let rng = Rng::new();
    u64::from(rng.random()) << 32 | u64::from(rng.random())
}
