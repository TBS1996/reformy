#!/usr/bin/env bash
 
cargo expand --bin main > exp.rs
cargo run --bin exp

