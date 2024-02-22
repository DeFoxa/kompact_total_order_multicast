use crate::master_types::*;
use anyhow::Result;
use kompact::prelude::*;
use rand::Rng;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BinaryHeap},
    time::{SystemTime, UNIX_EPOCH},
};
// TODO: Write logic for generating RFP from priority_queue sequence_numbers
// and updating state with accepted_proposal

#[derive(Debug, Clone, Eq)]
pub struct BroadcastMessage {
    worker_id: u8,
    sequence_number: i64,
    content: u8,
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
pub enum WorkerResponse {
    RfpResponse(RfpResponse),
    // NOTE: StateUpdateConfirmed: include worker_id num and logical_time from associated rfp
    StateUpdateConfirmed {
        worker_id: u8,
        logical_time: LamportClock,
    },
    NoResponse,
}
#[derive(Debug, Clone)]
pub struct Proposal {
    logical_time: LamportClock,
    proposal: BroadcastMessage,
}

#[derive(Debug, Clone)]
pub struct RfpResponse {
    proposed_message: Proposal,
}

#[derive(Debug, Clone)]
pub struct StateUpdate {
    seq_number: i32,
    message: u8,
}

#[derive(ComponentDefinition)]
pub struct Worker {
    ctx: ComponentContext<Self>,
    worker_id: u8,
    state: (u8, u8),
    priority_queue: BinaryHeap<BroadcastMessage>,
    delivered_messages: BTreeMap<LamportClock, u8>,
    message_port: ProvidedPort<MessagePort>,
}
// ignore_lifecycle!(Worker);

impl Worker {
    pub fn new(id: u8) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            worker_id: id,
            state: (0, 0),
            priority_queue: BinaryHeap::new(),
            delivered_messages: BTreeMap::new(),
            message_port: ProvidedPort::uninitialised(),
        }
    }
    /// This method is called at worker start, it generates the mock messages that are then added
    /// to the binaryheap for later proposals/broadcasts. seq_number gen is based on
    /// timestamp, worker_id and an index for spacing of the sequence_numbers - called with
    /// generate_sequence_number method.
    fn initialize_message_queue(&mut self) -> Result<()> {
        let mut rng = rand::thread_rng();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration since error")
            .as_millis();

        for i in 0..10 {
            let seq_num = self.generate_sequence_number(current_time, i);
            let msg_value: u8 = rng.gen_range(0..=25).into();
            let seq_num = self.generate_sequence_number(current_time, i).try_into()?;

            let msg = self.priority_queue.push(BroadcastMessage {
                worker_id: self.worker_id,
                sequence_number: seq_num,
                content: msg_value,
                deliverable: false,
            });
        }
        Ok(())
    }
    fn generate_sequence_number(&self, timestamp: u128, index: u64) -> i128 {
        (timestamp as i128) * 1000 + (self.worker_id as i128) * 100 + (index as i128)
    }

    /// Method Updates state from accepted_proposal from sender (master)
    fn update_state(&mut self, msg_content: u8) {
        // step 1: add msg_content: u8 to current state element at index 0
        let updated_element = (self.state.0 + msg_content) % 100;
        // step 2: rotate state.0 and state.1
        self.state = (self.state.1, updated_element);
    }

    fn generate_rfp_response(&mut self) -> Result<WorkerResponse> {
        //TODO: pull Reverse BinaryHeap for lowest timestamp in BH, generate proposal with
        //logical_time from rfp and associated BroadcastMessage
        let res = todo!();
        Ok(WorkerResponse::RfpResponse(res))
    }

    fn handle_accepted_proposal(
        &mut self,
        seq_number: i64,
        message: u8,
        logical_time: LamportClock,
    ) -> Result<WorkerResponse> {
        // TODO search if accepted proposal matches sequence number of entries in BH, if yes, mark
        // delivered, deliver then pop from BH
        self.update_state(message);
        //TODO: Log delivered message into Btreemap ordered by logical_time
        todo!();
        let id = self.worker_id;
        Ok(WorkerResponse::StateUpdateConfirmed {
            worker_id: self.worker_id,
            logical_time,
        })
    }

    fn handle_master_message(&mut self, msg: MasterMessage) -> Result<WorkerResponse> {
        match msg {
            MasterMessage::Rfp { master_clock } => {
                Ok(self.generate_rfp_response()?)
                //TODO: send res back to master through port handle
            }

            MasterMessage::AcceptedProposalBroadcast {
                seq_number,
                message,
                logical_time,
            } => {
                Ok(self.handle_accepted_proposal(seq_number, message, logical_time)?)
                //TODO: send StateUpdateConfirmation, include worker_id and logical_time,  back to master through port handle
            }
        }
    }
}

impl Actor for Worker {
    type Message = MasterMessage;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        self.handle_master_message(msg);

        Handled::Ok
    }
    fn receive_network(&mut self, _msg: NetMessage) -> Handled {
        unimplemented!("No receive network message handling on Worker")
    }
}
impl ComponentLifecycle for Worker {
    fn on_start(&mut self) -> Handled {
        // on start generate a set of sequence_num/Broadcast messages using random elements and add
        // to the binaryheap/btreemap, these will be our mock messages. RFP will not instigate the
        // generatio of new mock messages, the worker will just pull from the binary heap and then
        // reorder the heap with accepted sequence numbers, until every worker heap is empty, then
        // the processes complete and shutdown
        self.initialize_message_queue();
        Handled::Ok
    }
    fn on_stop(&mut self) -> Handled {
        Handled::Ok
    }
    fn on_kill(&mut self) -> Handled {
        self.on_stop()
    }
}

impl Provide<MessagePort> for Worker {
    fn handle(&mut self, event: MasterMessage) -> Handled {
        let res = self.handle_master_message(event);
        //TODO send response back to master

        Handled::Ok
    }
}
