// Timing harness.
//
// Unlike the `schema_policy` benchmark, nothing here touches I/O -- every
// measured operation is pure CPU work in-process, so the optimizer is free to
// hoist or delete a call whose result is unused. Every operation therefore goes
// through `black_box`.
//
// Rendering cost spans three orders of magnitude across the sweep (a 3-leaf
// payload vs. a 3002-leaf one), so a fixed repetition count would either be
// noise at the small end or take minutes at the large end. Instead a short
// warmup doubles as a cost probe, and the repetition count is chosen from it to
// fill a fixed time budget.

use std::hint::black_box;
use std::time::{Duration, Instant};

const WARMUP: usize = 100; // untimed repetitions, also used to estimate cost.
const MIN_REPS: usize = 100;
const MAX_REPS: usize = 500_000;

// Target time per measurement. Public so reports can quote it without drifting
// from the value actually used.
pub const DEFAULT_BUDGET: Duration = Duration::from_millis(2000);

pub struct Timing {
    pub avg_us: f64,
    pub reps: usize,
}

// Run `op` repeatedly and report its average latency in microseconds.
pub fn time<T, F: FnMut() -> T>(op: F) -> Timing {
    time_with(DEFAULT_BUDGET, op)
}

// As `time`, with an explicit time budget. An HTTP round trip costs orders of
// magnitude more than a bare render and carries more jitter, so the end-to-end
// experiment can afford to spend longer per measurement.
pub fn time_with<T, F: FnMut() -> T>(budget: Duration, mut op: F) -> Timing {
    // Warm up (caches, branch predictors, allocator arenas) and measure roughly
    // what one call costs.
    let probe = Instant::now();
    for _ in 0..WARMUP {
        black_box(op());
    }
    let per_op = probe.elapsed().as_secs_f64() / WARMUP as f64;

    // Spend about `budget` on the real measurement.
    let reps = if per_op > 0.0 {
        (budget.as_secs_f64() / per_op) as usize
    } else {
        MAX_REPS
    };
    let reps = reps.clamp(MIN_REPS, MAX_REPS);

    let start = Instant::now();
    for _ in 0..reps {
        black_box(op());
    }
    let avg_us = start.elapsed().as_secs_f64() * 1e6 / reps as f64;

    Timing { avg_us, reps }
}
