//! Advanced bench modes (watch, regression, compare-versions, live, matrix).

pub(crate) mod matrix;
pub(crate) mod regression;
pub(crate) mod watch;

pub(crate) use matrix::{run_bench_live, run_bench_matrix};
pub(crate) use regression::{run_bench_compare_versions, run_bench_regression};
pub(crate) use watch::run_bench_watch;
