use kompact::prelude::*;

#[derive(Debug, Clone)]
pub struct WorkerResponse {
    proposed_sequence_number: i32,
    msg: i32,
}
