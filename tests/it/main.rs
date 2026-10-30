//! The integration tests, one crate: each module checks tddlog's answers
//! through the library or the binary against ProbLog's, enumeration's or
//! hand-computed ones, except `front_pages`, which checks the front pages'
//! links into the guide. `cargo test --test it NAME` runs the tests whose path
//! holds NAME (`layouts::grid`, say).

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
mod flexible;
mod front_pages;
mod guards;
mod implicit_levels;
mod indicators;
mod input;
mod link_reuse;
mod long_bodies;
mod naming;
mod narrowed;
mod negations;
mod networks;
mod or_route;
mod problog_suite;
mod query_count;
mod query_rules;
mod renormalize;
mod separator_copies;
mod share;
mod shared_operand;
mod table_models;
mod templates;
mod terms;
mod ties;
mod topdown;
mod undefined;
mod underflow;
mod writings;
