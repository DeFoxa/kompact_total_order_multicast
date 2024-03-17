use crate::master_types::*;
use anyhow::{anyhow, Result};
use kompact::prelude::*;
use rand::Rng;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BinaryHeap},
    time::{SystemTime, UNIX_EPOCH},
};

//TODO: Rewrite generation of binary heap sequence_numbers and messages at initialization

//TODO: add logging and debugging

#[derive(Debug, Clone, Eq)]
pub struct BroadcastMessage {
    pub worker_id: u8,
    pub sequence_number: i64,
    pub content: u8,
    pub deliverable: bool,
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
    // NOTE: StateUpdateConfirmed: include worker_id and logical_time from associated rfp
    StateUpdateConfirmed {
        worker_id: u8,
        logical_time: LamportClock,
    },
    MessageQueueEmpty,
    NoResponse,
}
#[derive(Debug, Clone)]
pub struct Proposal {
    pub logical_time: LamportClock,
    pub worker_id: u8,
    pub proposal: BroadcastMessage,
}

#[derive(Debug, Clone)]
pub struct RfpResponse {
    pub proposed_message: Proposal,
}

#[derive(Debug, Clone)]
pub struct StateUpdate {
    seq_number: i32,
    message: u8,
}

///Note: undelivered_priority_queue implementation a Min heap using cmp::Reverse, for ordering by lowest
///sequence number (rfp proposal)
#[derive(ComponentDefinition)]
pub struct Worker {
    ctx: ComponentContext<Self>,
    worker_id: u8,
    state: (u8, u8),
    undelivered_priority_queue: BinaryHeap<Reverse<BroadcastMessage>>,
    deliverable_queue: BTreeMap<LamportClock, BroadcastMessage>,
    delivered_messages: BTreeMap<LamportClock, BroadcastMessage>,
    message_port: ProvidedPort<MessagePort>,
}
// ignore_lifecycle!(Worker);

impl Worker {
    pub fn new(id: u8) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            worker_id: id,
            state: (0, 0),
            undelivered_priority_queue: BinaryHeap::new(),
            deliverable_queue: BTreeMap::new(),
            delivered_messages: BTreeMap::new(),
            message_port: ProvidedPort::uninitialised(),
        }
    }
    /// Method is called at worker start, it generates the mock messages that are then added
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

            self.undelivered_priority_queue
                .push(Reverse(BroadcastMessage {
                    worker_id: self.worker_id,
                    sequence_number: seq_num,
                    content: msg_value,
                    deliverable: false,
                }));
        }
        Ok(())
    }
    /// TODO: Refactor sequence_number generation relative to notes
    fn generate_sequence_number(&self, timestamp: u128, index: u64) -> i128 {
        (timestamp as i128) * 1000 + (self.worker_id as i128) * 100 + (index as i128)
    }

    /// Method Updates state from accepted_proposal from sender (master)
    fn update_state(&mut self, msg_content: u8) {
        // step 1: add msg_content mod 100: u8 to current state element at index 0
        let updated_element = (self.state.0 + msg_content) % 100;
        // step 2: rotate state.0 and state.1
        self.state = (self.state.1, updated_element);
    }

    fn generate_rfp_response(&mut self, rfp_logical_time: LamportClock) -> Result<WorkerResponse> {
        //TODO: pull Reverse BinaryHeap for lowest timestamp in BH, generate proposal with
        //logical_time from rfp and associated BroadcastMessage
        if let Some(Reverse(top_of_queue)) = self.undelivered_priority_queue.peek() {
            let proposal = Proposal {
                logical_time: rfp_logical_time,
                worker_id: self.worker_id,
                proposal: top_of_queue.clone(),
            };

            let response = WorkerResponse::RfpResponse(RfpResponse {
                proposed_message: proposal,
            });
            Ok(response)
        } else {
            info!(
                self.ctx.log(),
                "priority queue empty, can't generate rfp response"
            );
            Ok(WorkerResponse::MessageQueueEmpty)
        }
    }

    fn handle_accepted_proposal(
        &mut self,
        logical_time: LamportClock,
        broadcast_message: BroadcastMessage,
    ) -> Result<WorkerResponse> {
        /// Checks if current worker generated the accepted proposal, if true pop BroadcastMessage
        /// message off of undelivered_priority_queue, mark as deliverable -> add to deliverable
        /// queue -> run logic to verify proper ordering of message delivery -> deliver. Else
        /// directly queue message in deliverable_queue -> run logic for proper ordering -> update
        /// state and deliver
        match self.undelivered_priority_queue.peek() {
            Some(top_of_queue)
                if top_of_queue.0.sequence_number == broadcast_message.sequence_number
                    || broadcast_message.worker_id == self.worker_id =>
            {
                let mut deliverable_message = self.undelivered_priority_queue.pop().unwrap();
                deliverable_message.0.deliverable = true;
                let result = self.queue_message_or_deliver(logical_time, deliverable_message.0)?;
                Ok(result)
            }
            _ => {
                let result = self.queue_message_or_deliver(logical_time, broadcast_message)?;
                Ok(result)
            }
        }
    }
    fn queue_message_or_deliver(
        &mut self,
        logical_time: LamportClock,
        broadcast_message: BroadcastMessage,
    ) -> Result<WorkerResponse> {
        if let Some(last_delivered) = self.delivered_messages.iter().max_by_key(|p| p.0) {
            if last_delivered.0.time + 1 == logical_time.time {
                self.update_state(broadcast_message.content);
                self.delivered_messages
                    .insert(logical_time, broadcast_message);
                // let new_clock = logical_time.time + 1;
                self.recursively_process_queue_for_sequential_deliverables(LamportClock {
                    time: logical_time.time + 1,
                })?
            } else {
                self.deliverable_queue
                    .insert(logical_time, broadcast_message);
            }
        }
        Ok(WorkerResponse::StateUpdateConfirmed {
            worker_id: self.worker_id,
            logical_time,
        })
    }

    fn recursively_process_queue_for_sequential_deliverables(
        &mut self,
        logical_time: LamportClock,
    ) -> Result<()> {
        match self.check_queue_for_deliverable(logical_time) {
            Ok(true) => {
                self.recursively_process_queue_for_sequential_deliverables(LamportClock {
                    time: logical_time.time + 1,
                })?;
            }
            Ok(false) => return Ok(()),
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }
    fn check_queue_for_deliverable(&mut self, logical_time: LamportClock) -> Result<bool> {
        let next_deliverable = LamportClock {
            time: logical_time.time + 1,
        };

        if let Some((key, deliverable)) = self.deliverable_queue.remove_entry(&next_deliverable) {
            self.update_state(deliverable.content);
            self.delivered_messages.insert(key, deliverable.clone());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn handle_master_message(&mut self, msg: MasterMessage) -> Result<WorkerResponse> {
        match msg {
            MasterMessage::Rfp { master_clock } => {
                Ok(self.generate_rfp_response(master_clock)?)
                //TODO: send res back to master through port handle
            }

            MasterMessage::AcceptedProposalBroadcast {
                logical_time,
                broadcast_message,
            } => {
                Ok(self.handle_accepted_proposal(logical_time, broadcast_message)?)
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
        // to the binaryheap these will be our mock messages. RFP will not instigate the
        // generatio of new mock messages, the worker will just pull from the bh and then
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
        let res = self.handle_master_message(event).unwrap();
        self.message_port.trigger(res);

        Handled::Ok
    }
}
