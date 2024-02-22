#![allow(warnings)]
use kompact::prelude::*;

pub mod errors;
pub mod kompact_logic;
pub mod master_types;
pub mod worker_types;

//TODO: consider thread pool allocations default vs explicit config, dynamic adjustment
//should be unnecessary for this program. Although, KompactConfig::default should suffice.
fn main() {}
