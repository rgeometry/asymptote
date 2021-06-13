use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Display;
use std::time::Duration;
use std::time::Instant;

static TIME_LIMIT_SINGLE: Duration = Duration::from_secs(10);
static TIME_LIMIT_ALGO: Duration = Duration::from_secs(30);

// Parameters:
//   Time per benchmark.
//   N increment factor.

pub fn asymptote<A, B>(
    name: &str,
    components: &[Component],
    gen: impl Fn(u64) -> A + Copy,
    runner: impl Fn(A) -> B + Copy,
) {
    let mut data = Vec::new();
    for datum in bench_algo(gen, runner) {
        data.push(datum);
        if data.len() > 2 {
            let results = analyze(components.into(), &data);
            print!("{:<30} {}\r", format!("{}:", name), results);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }
    println!();
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Debug)]
pub enum Component {
    N,
    N2,
    LogN,
    NLogN,
}
use Component::*;

impl Component {
    fn key(&self) -> &'static str {
        match self {
            N => "X_N",
            N2 => "X_N2",
            LogN => "X_LOGN",
            NLogN => "X_NLOGN",
        }
    }
}
impl Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            N => write!(f, "n"),
            N2 => write!(f, "n²"),
            LogN => write!(f, "logn"),
            NLogN => write!(f, "nlogn"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Results {
    pub rsquared: f64,
    pub constant: f64,
    pub components: BTreeMap<Component, f64>,
}

impl Display for Results {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tmp = String::new();
        for (&component, &ns) in self.components.iter() {
            let d = Duration::from_nanos(ns.round() as u64);
            tmp += &format!("{:?} {} ", d, component);
        }
        write!(f, "{:<20} R² = {:.2}", tmp, self.rsquared)
    }
}

pub fn analyze(mut components: Vec<Component>, data: &[(u64, u64)]) -> Results {
    components.sort_unstable();
    components.dedup();
    let y: Vec<f64> = data.iter().map(|&(_x, y)| y as f64).collect();
    let x: Vec<f64> = data.iter().map(|&(x, _y)| x as f64).collect();
    let x_lin: Vec<f64> = x.iter().copied().collect();
    let x_logn: Vec<f64> = x.iter().map(|&x| x.log2()).collect();
    let x_nlogn: Vec<f64> = x.iter().map(|&x| x * x.log2()).collect();
    let x_two: Vec<f64> = x.iter().map(|&x| x * x).collect();

    let data = vec![
        ("Y", y),
        (N.key(), x_lin),
        (N2.key(), x_two),
        (LogN.key(), x_logn),
        (NLogN.key(), x_nlogn),
    ];
    let mut formula = String::new();
    formula += "Y ~ ";
    for (nth, component) in components.iter().enumerate() {
        if nth != 0 {
            formula += " + ";
        }
        formula += component.key();
    }
    let data = RegressionDataBuilder::new().build_from(data).unwrap();
    let model = FormulaRegressionBuilder::new()
        .data(&data)
        .formula(formula)
        .fit()
        .unwrap();
    // dbg!(&model);
    let params = model.parameters;
    let mut out = BTreeMap::new();
    for (nth, component) in components.into_iter().enumerate() {
        out.insert(component, params.regressor_values[nth]);
    }
    Results {
        rsquared: model.rsquared,
        constant: params.intercept_value,
        components: out,
    }
}
// y = [1 x log(n) x^2 ]
// (X.t() X).inv() * X.t() * y

// fn bench_to_disk<A, B>(name: &str, gen: impl Fn(u64) -> A + Copy, runner: impl Fn(A) -> B + Copy) {
//     let results = bench_algo(gen, runner);
//     todo!()
// }

pub fn bench_algo<A, B>(
    gen: impl Fn(u64) -> A + Copy,
    runner: impl Fn(A) -> B + Copy,
) -> impl Iterator<Item = (u64, u64)> {
    let start = Instant::now();
    let mut n = 1;
    std::iter::from_fn(move || {
        if start.elapsed() < TIME_LIMIT_ALGO {
            let result = (n, bench_stable(|| gen(n), runner));
            // n *= 2;
            // n += 1;
            n = ((n as f64) * 1.1).ceil() as u64;
            Some(result)
        } else {
            None
        }
    })
    // while start.elapsed() < TIME_LIMIT_ALGO {
    //     let result = bench_stable(|| gen(n), runner);
    //     results.push((n, result));
    //     // use std::io::Write;
    //     // println!("{}/{}", n, result);
    //     // std::io::stdout().flush().unwrap();
    //     // dbg!(n, result);
    //     n *= 2;
    // }
    // results
}

#[inline]
// Run a function until two sequences have a difference of less than 5%.
fn bench_stable<A, B>(gen: impl Fn() -> A + Copy, runner: impl Fn(A) -> B + Copy) -> u64 {
    let factor = 1.5f64;
    let mut n = 10usize;
    let mut last_run = bench(n, gen, runner);

    while last_run < TIME_LIMIT_SINGLE.as_nanos() as u64 {
        // dbg!(n, last_run);
        let this_n = (n as f64 * factor).ceil() as usize;
        let this_run = bench(this_n, gen, runner);
        let difference = (1f64 - (last_run as f64 * factor) / (this_run as f64)).abs();
        // dbg!(difference);
        if difference < 0.05 {
            return this_run / this_n as u64;
        }
        last_run = this_run;
        n = this_n;
    }
    last_run / n as u64
}

#[inline]
// Run a function N times and record how many nanoseconds elapsed.
fn bench<A, B>(n: usize, gen: impl Fn() -> A, runner: impl Fn(A) -> B) -> u64 {
    let mut vec = Vec::with_capacity(n);
    for _ in 0..n {
        vec.push(gen());
    }
    let mut out = Vec::with_capacity(n);
    let now = Instant::now();
    vec.into_iter().for_each(|a| out.push(black_box(runner(a))));
    let elapsed = now.elapsed();
    std::mem::drop(out);
    elapsed.as_nanos() as u64
}

fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}
