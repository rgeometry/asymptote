use asymptote::Component::*;
use asymptote::*;
use std::iter::FromIterator;

static EPSILON: f64 = 1e-3;

fn main() {
    linear();
    linear_constant();
    linear_noise();
    not_linear();
    complex();
}

fn linear() {
    let data: Vec<(u64, u64)> = Vec::from_iter((1..1000).map(|x| {
        let y = x;
        (x, y)
    }));
    let results = analyze(vec![N], &data);
    dbg!(&results);
    assert!(approx_eq(results.components[&N], 1.0));
}

fn linear_constant() {
    let data: Vec<(u64, u64)> = Vec::from_iter((1..1000).map(|x| {
        let y = x + 25;
        (x, y)
    }));
    let results = analyze(vec![N], &data);
    dbg!(&results);
    assert!(approx_eq(results.components[&N], 1.0));
    assert!(approx_eq(results.constant, 25.0));
}

fn linear_noise() {
    let data: Vec<(u64, u64)> = Vec::from_iter((1..1000).map(|x| {
        let y = x + x % 5;
        (x, y)
    }));
    let results = analyze(vec![N], &data);
    dbg!(&results);
    assert!(approx_eq(results.components[&N], 1.0));
}

fn not_linear() {
    let data: Vec<(u64, u64)> = Vec::from_iter((1..1000).map(|x| {
        let y = x * x;
        (x, y)
    }));
    let results = analyze(vec![N], &data);
    dbg!(&results);
    assert!(results.rsquared < 0.95);
}

fn complex() {
    let data: Vec<(u64, u64)> = Vec::from_iter((1..10000).map(|x| {
        let y = 50 + 10 * x + x * x + ((x as f64).log2() as u64);
        (x, y)
    }));
    let results = analyze(vec![N, N2, LogN], &data);
    dbg!(&results);
    // assert!(approx_eq(results.components[&N], 1.0));
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}
