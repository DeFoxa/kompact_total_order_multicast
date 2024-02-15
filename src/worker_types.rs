use crate::master_types::*;
use kompact::prelude::*;
use rand::Rng;
use std::{
    cmp::Ordering,
    collections::{btree_map, BTreeMap, BinaryHeap},
};

//NOTE: Writing priority queue as both binary heap and Btreemap, because im curious about
//implementation, performance and testing both versions. For actual testing, comment out the unused type
//and run without the overhead of managing both types, I'll write methods for both.

#[derive(Debug, Clone, Eq)]
pub struct BroadcastMessage {
    sequence_number: i32,
    content: usize,
    deliverable: bool,
}

impl PartialEq for BroadcastMessage {
    fn eq(&self, other: &Self) -> bool {
        self.sequence_number == other.sequence_number
    }
}
impl Ord for BroadcastMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}
impl PartialOrd for BroadcastMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub enum WorkerMessages {
    External(External),
    InternalStateUpdate(StateUpdate),
}

impl From<MasterMessage> for WorkerMessages {
    fn from(item: MasterMessage) -> Self {
        match item {
            MasterMessage::Rfp => WorkerMessages::External(External::MasterMessage(item)),
            MasterMessage::AcceptedProposalBroadcast {
                seq_number,
                message,
            } => WorkerMessages::External(External::MasterMessage(item)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum External {
    WorkerResponse(WorkerResponse),
    MasterMessage(MasterMessage),
}

#[derive(Debug, Clone)]
pub enum WorkerResponse {
    RfpResponse(RfpResponse),
    StateUpdateConfirmed,
    NoResponse,
    // NOTE: acknowledgement mechanism as response to AcceptedProposalBroadcast from master
    // based on logic, master can then shutdown workers or send next rfp iteration when
    // received confirmations = num_workers
}

#[derive(Debug, Clone)]
pub struct RfpResponse {
    proposed_message: BroadcastMessage,
}

#[derive(Debug, Clone)]
pub struct StateUpdate {
    seq_number: i32,
    message: u8,
}

#[derive(ComponentDefinition)]
pub struct Worker {
    ctx: ComponentContext<Self>,
    state: (u8, u8),
    /// priority_queue as BinaryHeap
    priority_queue: BinaryHeap<BroadcastMessage>,
    ///priority queue as btreemap sorted by seq_number
    priority_btree: BTreeMap<i32, BroadcastMessage>,
    message_port: ProvidedPort<MessagePort>,
}
ignore_lifecycle!(Worker);

impl Worker {
    pub fn new() -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            state: (0, 0),
            priority_queue: BinaryHeap::new(),
            priority_btree: BTreeMap::new(),
            message_port: ProvidedPort::uninitialised(),
        }
    }
    fn generate_message(&mut self) -> u8 {
        let mut rng = rand::thread_rng();
        let msg_value: u8 = rng.gen_range(0..=25);
        msg_value
    }

    fn update_state(&mut self, seq_number: i32) {
        todo!();
    }
    fn update_state_internal_message(&mut self, seq_number: i32, msg: u8) {
        //NOTE: updating state through internal message passing, if message received via direct
        //actor message passing, instead of port
        todo!();
    }

    fn handle_worker_response(&mut self) -> WorkerResponse {
        todo!();
        WorkerResponse::NoResponse
    }

    fn generate_rfp(&mut self) -> WorkerResponse {
        let res = todo!();
        WorkerResponse::RfpResponse(res)
    }

    fn handle_accepted_proposal(&mut self, seq_number: i32, message: u8) -> WorkerResponse {
        self.update_state(seq_number);
        todo!();
        WorkerResponse::StateUpdateConfirmed
    }

    fn handle_external(&mut self, msg: External) {
        match msg {
            External::MasterMessage(m) => self.handle_master_message(m),
            External::WorkerResponse(m) => self.handle_worker_response(),
        };
    }

    fn handle_master_message(&mut self, msg: MasterMessage) -> WorkerResponse {
        match msg {
            MasterMessage::Rfp => self.generate_rfp(),

            MasterMessage::AcceptedProposalBroadcast {
                seq_number,
                message,
            } => self.handle_accepted_proposal(seq_number, message),
        }
    }
}

impl Actor for Worker {
    type Message = WorkerMessages;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        match msg {
            WorkerMessages::External(master_request) => self.handle_external(master_request),
            WorkerMessages::InternalStateUpdate(state_update) => {
                self.update_state_internal_message(state_update.seq_number, state_update.message);
            }
        }

        Handled::Ok
    }
    fn receive_network(&mut self, _msg: NetMessage) -> Handled {
        unimplemented!("No receive network message handling on Worker")
    }
}

impl Provide<MessagePort> for Worker {
    fn handle(&mut self, event: MasterMessage) -> Handled {
        self.handle_master_message(event);

        Handled::Ok
    }
}
