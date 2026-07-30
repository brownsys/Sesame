// Micro-benchmark: cost of constructing and attaching schema policies to DB
// query results.
//
// Commit affd3ec ("perf(mysql): resolve schema policies once per query instead
// of per cell") replaced a per-cell registry lookup with a per-query snapshot.
// This benchmark measures the effect by running the same query+iterate+consume
// flow against three connection strategies:
//
//   * plain - raw `mysql` connector, no Sesame policies at all (the floor).
//   * old   - the pre-commit behaviour, reconstructed by calling the still-
//             public `get_schema_policies` once per cell.
//   * new   - the current `sesame_mysql` path: policies snapshotted once per
//             query and shared by every row.
//
// See README.md for details, and the modules below for the pieces:
//   policy - the schema policy under test.
//   db     - connecting, priming, and building the query.
//   load   - the three flows, timing, and the sweep/reporting.

mod db;
mod load;
mod policy;

const TABLE_ROWS: usize = 10000; // rows written to the table during priming.
const KS: &[usize] = &[1, 10, 100, 1000]; // result-set sizes to sweep.

fn main() {
    println!("Priming `{}` with {} rows...", db::DB_NAME, TABLE_ROWS);
    db::prime(TABLE_ROWS);

    let results = load::run_sweep(KS);
    load::print_table(&results);
}
