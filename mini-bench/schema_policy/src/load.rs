// The benchmark loads: the three query+iterate+consume flows, the timing
// harness, and the sweep that runs them and cross-checks their results.

use std::time::Instant;

use mysql::prelude::Queryable;
use mysql::{Conn, Row, Value};

use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{from_value, PConValue, SesameConn};

use crate::db::{self, SUM_COLUMN};

const REPS: usize = 1000; // timed repetitions per (variant, k).
const WARMUP: usize = 20; // untimed warmup repetitions.

// Average latency (microseconds) of one query+iterate+consume for each variant
// at a given result-set size. Two access methods are compared: `unwrap`
// materialises every cell of the row, `get` fetches only the summed column.
pub struct Measurement {
    pub k: usize,
    pub plain_unwrap_us: f64,
    pub new_unwrap_us: f64,
    pub plain_get_us: f64,
    pub new_get_us: f64,
}

// --- The flows. Each returns (row count, summed column). ---

// Sesame, unwrap: PConRow::unwrap builds a PCon for every cell, attaching each
// cell's policy via the per-query snapshot. This is the optimized path.
fn run_new_unwrap(conn: &mut SesameConn, k: usize) -> (u64, i64) {
    let result = conn.query_iter(db::select(k)).unwrap();
    let mut count = 0u64;
    let mut sum = 0i64;
    for row in result {
        let row = row.unwrap();
        let mut cells: Vec<PConValue> = row.unwrap();
        count += 1;
        sum += reveal_no_policy(cells.swap_remove(SUM_COLUMN));
    }
    (count, sum)
}

// Plain mysql, unwrap: materialise the raw cell vector, read the summed column.
fn run_plain_unwrap(conn: &mut Conn, k: usize) -> (u64, i64) {
    let result = conn.query_iter(db::select(k)).unwrap();
    let mut count = 0u64;
    let mut sum = 0i64;
    for row in result {
        let row: Row = row.unwrap();
        let vals: Vec<Value> = row.unwrap();
        count += 1;
        sum += mysql::from_value::<i64>(vals[SUM_COLUMN].clone());
    }
    (count, sum)
}

// Sesame, get: fetch only the summed column. PConRow::get attaches just that
// cell's policy (NoPolicy here, since SUM_COLUMN carries no schema policy) via
// the snapshot. Requesting the cell as a mysql::Value (identity FromValue)
// yields a PConValue so the same reveal path applies.
fn run_new_get(conn: &mut SesameConn, k: usize) -> (u64, i64) {
    let result = conn.query_iter(db::select(k)).unwrap();
    let mut count = 0u64;
    let mut sum = 0i64;
    for row in result {
        let row = row.unwrap();
        let cell: PConValue = row.get(SUM_COLUMN).unwrap();
        count += 1;
        sum += reveal_no_policy(cell);
    }
    (count, sum)
}

// Plain mysql, get: fetch and convert only the summed column directly.
fn run_plain_get(conn: &mut Conn, k: usize) -> (u64, i64) {
    let result = conn.query_iter(db::select(k)).unwrap();
    let mut count = 0u64;
    let mut sum = 0i64;
    for row in result {
        let row: Row = row.unwrap();
        count += 1;
        sum += row.get::<i64, _>(SUM_COLUMN).unwrap();
    }
    (count, sum)
}

// Reveal a cell known to carry NoPolicy (the summed column has no registered
// schema policy) by specializing it back to NoPolicy and discarding the box.
fn reveal_no_policy(cell: PConValue) -> i64 {
    let pcon: PCon<i64, NoPolicy> = from_value(cell).unwrap();
    pcon.discard_box()
}

// --- Timing harness. ---

fn time<F: FnMut() -> (u64, i64)>(mut flow: F) -> (f64, (u64, i64)) {
    let mut last = (0u64, 0i64);
    for _ in 0..WARMUP {
        last = flow();
    }
    let start = Instant::now();
    for _ in 0..REPS {
        last = flow();
    }
    let avg_us = start.elapsed().as_secs_f64() * 1e6 / REPS as f64;
    (avg_us, last)
}

// --- Sweep + reporting. ---

// Run all four flows for each k, asserting they agree, and collect timings.
pub fn run_sweep(ks: &[usize]) -> Vec<Measurement> {
    let mut sesame_conn = db::connect_sesame();
    let mut plain_conn = db::connect_plain();

    let mut out = Vec::with_capacity(ks.len());
    for &k in ks {
        let (plain_unwrap_us, ans) = time(|| run_plain_unwrap(&mut plain_conn, k));
        let (new_unwrap_us, a) = time(|| run_new_unwrap(&mut sesame_conn, k));
        assert_eq!(a, ans, "new/unwrap vs plain/unwrap mismatch at k={}", k);
        let (plain_get_us, a) = time(|| run_plain_get(&mut plain_conn, k));
        assert_eq!(a, ans, "plain/get vs plain/unwrap mismatch at k={}", k);
        let (new_get_us, a) = time(|| run_new_get(&mut sesame_conn, k));
        assert_eq!(a, ans, "new/get vs plain/unwrap mismatch at k={}", k);

        // Every flow must see the same k rows and the same column sum.
        assert_eq!(ans.0 as usize, k, "plain returned wrong row count");

        out.push(Measurement {
            k,
            plain_unwrap_us,
            new_unwrap_us,
            plain_get_us,
            new_get_us,
        });
    }
    out
}

pub fn print_table(rows: &[Measurement]) {
    println!(
        "Averaging over {} reps ({} warmup) per cell.\n",
        REPS, WARMUP
    );
    println!(
        "  {:>5} | {:>11} {:>11} {:>9} | {:>11} {:>11} {:>9}",
        "", "plain", "new", "new/plain", "plain", "new", "new/plain"
    );
    println!(
        "  {:>5} | {:^33} | {:^33}",
        "k", "unwrap: all cells (us)", "get: summed cell (us)"
    );
    println!(
        "  {:->5}-+-{:->33}-+-{:->33}",
        "", "", ""
    );
    for r in rows {
        println!(
            "  {:>5} | {:>11.2} {:>11.2} {:>8.2}x | {:>11.2} {:>11.2} {:>8.2}x",
            r.k,
            r.plain_unwrap_us,
            r.new_unwrap_us,
            r.new_unwrap_us / r.plain_unwrap_us,
            r.plain_get_us,
            r.new_get_us,
            r.new_get_us / r.plain_get_us,
        );
    }
    println!(
        "\nnew/plain = residual Sesame overhead over the no-policy floor.\n`unwrap` materialises every cell; `get` fetches only the summed column\n(which carries no policy). Validated: all flows agree on (row count,\nsummed column) at every k."
    );
}
