//! The integration tests, one crate: each module checks tddlog's answers
//! through the library or the binary against ProbLog's, enumeration's or
//! hand-computed ones. `cargo test --test it NAME` runs the tests whose
//! path holds NAME (`layouts::grid`, say).

mod common;
mod layouts;

mod ad_tails;
mod binary;
mod binary_parent;
mod cli;
mod contexts;
mod dense_count;
mod evidence_probability;
mod evidence_walk;
mod examples;
mod guards;
mod implicit_levels;
mod indicators;
mod link_reuse;
mod naming;
mod narrowed;
mod negations;
mod or_route;
mod problog_suite;
mod query_count;
mod renormalize;
mod separator_copies;
mod share;
mod shared_operand;
mod table_models;
mod templates;
mod ties;
mod underflow;
