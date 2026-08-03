// Shared setup for the template-render benchmarks. The payload structs, their
// policy, the data generation, and the render helpers all live here so that the
// standalone and end-to-end experiments measure literally the same work.

pub mod common;
pub mod server;
