use asymptote::Component::*;
use asymptote::*;
use rand::prelude::SliceRandom;
use std::iter::FromIterator;

fn main() {
    test_sort();
    test_sort_unstable();
}

fn gen_vec(n: u64) -> Vec<u64> {
    let mut vec = Vec::from_iter(0..n * 10);
    vec.shuffle(&mut rand::thread_rng());
    vec
}

fn test_sort() {
    asymptote("sort", &[NLogN], gen_vec, |mut v| {
        v.sort();
        v
    });
}

fn test_sort_unstable() {
    asymptote("sort_unstable", &[NLogN], gen_vec, |mut v| {
        v.sort_unstable();
        v
    });
}
