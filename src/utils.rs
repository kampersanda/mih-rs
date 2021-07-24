use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};

pub fn gen_random_codes<T>(size: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    let mut rng = thread_rng();
    let mut codes: Vec<T> = Vec::with_capacity(size);
    for _ in 0..size {
        codes.push(rng.gen::<T>());
    }
    codes
}
