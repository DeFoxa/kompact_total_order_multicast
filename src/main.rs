#![allow(warnings)]
use kompact::prelude::*;
use master_types::Master;
use worker_types::Worker;

pub mod errors;
pub mod master_types;
pub mod worker_types;

//TODO: consider thread pool allocations default vs explicit config
//dynamic adjustment should be unnecessary for this program. KompactConfig::default should suffice.

fn main() {
    let mut master = Master::new(4);
    // pull generated workers out of master
    // initialize message queue/worker
    // start rfp loop
}
