use crate::worker_types::*;
use anyhow::{anyhow, Result};
use futures::future::join;
use kompact::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::{
    cmp::Ordering,
    {thread, time},
};

// TODO: write handling for worker state_update_confirmed
// TODO: add async to applicable methods: message req/res between master/workers
// TODO:  write worker and master handling for tracking active workers on master
// side, so master has a way to verify active and inactive workers and handle state update
// confirmation and outstanding rfp
// TODO: Finish errors.rs structure, methods and implemented on master/worker
// TODO: see WorkerState comment

#[derive(ComponentDefinition)]
pub struct Master {
    ctx: ComponentContext<Self>,
    message_port: RequiredPort<MessagePort>,
    worker_states: HashMap<WorkerId, WorkerState>,
    worker_components: Vec<Arc<Component<Worker>>>,
    // NOTE: oustanding_proposals currently unused, keeping it until worker state management logic
    // finalized
    outstanding_proposals: Option<Ask<MasterMessage, WorkerResponse>>,
    worker_response: Vec<Option<WorkerResponse>>,
    worker_refs: Vec<ActorRefStrong<MasterMessage>>,
    logical_clock: LamportClock,
}

impl Master {
    fn new(num_workers: u8) -> Self {
        let mut worker_map: HashMap<WorkerId, WorkerState> = HashMap::new();
        for i in 0..num_workers {
            worker_map.insert(WorkerId(i), WorkerState::default());
        }

        Self {
            ctx: ComponentContext::uninitialised(),
            message_port: RequiredPort::uninitialised(),
            worker_states: worker_map,
            worker_components: Vec::with_capacity(num_workers.into()),
            outstanding_proposals: None,
            worker_response: Vec::with_capacity(num_workers.into()),
            worker_refs: Vec::with_capacity(num_workers.into()),
            logical_clock: LamportClock::new(),
        }
    }
    fn request_for_proposal(&mut self) {
        /// Adding jitter component to simulate network latency, will test handling of async
        /// message arrival
        let mut rng = rand::thread_rng();
        let jitter: u32 = rng.gen_range(25..100);
        let delay_duration = std::time::Duration::from_millis((jitter).into());
        let workers = self.worker_refs.clone();

        for _ in workers.iter() {
            self.schedule_once(delay_duration, move |new_self, _context| {
                new_self.message_port.trigger(MasterMessage::Rfp {
                    master_clock: new_self.logical_clock,
                });
                new_self.ctx().system().shutdown_async();
                Handled::Ok
            });
        }
    }
    fn filter_broadcast_propsals(&self) -> Result<Vec<(LamportClock, BroadcastMessage)>> {
        let filtered_proposals = self
            .worker_response
            .clone()
            .into_iter()
            .filter_map(|response| match response {
                Some(WorkerResponse::RfpResponse(rfp_response)) => Some((
                    rfp_response.proposed_message.logical_time,
                    rfp_response.proposed_message.proposal,
                )),
                _ => None,
            })
            .collect();
        Ok(filtered_proposals)
    }

    fn process_proposals(&self) -> Result<MasterMessage> {
        let proposals = self.filter_broadcast_propsals()?;
        let accepted = proposals.iter().min_by(|a, b| {
            a.1.sequence_number
                .cmp(&b.1.sequence_number)
                .then_with(|| a.1.worker_id.cmp(&b.1.worker_id))
        });

        match accepted {
            Some((lamport_clock, broadcast_message)) => {
                Ok(MasterMessage::AcceptedProposalBroadcast {
                    logical_time: lamport_clock.clone(),
                    broadcast_message: broadcast_message.clone(),
                })
            }
            None => {
                debug!(self.ctx.log(), "failed to determine proposal");
                Err(anyhow!("failed to determine accepted proposal"))
            }
        }
    }

    fn broadcast_accepted_proposal(&self, message: MasterMessage) {
        for worker in &self.worker_refs {
            worker.tell(message.clone());
        }
    }

    fn track_worker_status(&mut self, worker_id: WorkerId, state: WorkerState) {
        self.worker_states.insert(worker_id, state);
    }

    fn check_active_workers(&self) -> Vec<WorkerId> {
        self.worker_states
            .iter()
            .filter_map(|(worker_id, state)| {
                if let WorkerState::Active(_) = state {
                    Some(worker_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn check_possible_fault_workers(&self) -> Vec<WorkerId> {
        self.worker_states
            .iter()
            .filter_map(|(worker_id, state)| {
                if let WorkerState::PossibleFault = state {
                    Some(worker_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn check_dead_workers(&self) -> Vec<WorkerId> {
        self.worker_states
            .iter()
            .filter_map(|(worker_id, state)| {
                if let WorkerState::Dead = state {
                    Some(worker_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn check_non_active_workers(&self) -> Vec<(WorkerId, WorkerState)> {
        self.worker_states
            .iter()
            .filter_map(|(worker_id, state)| match state {
                WorkerState::PossibleFault | WorkerState::Dead => {
                    Some((worker_id.clone(), state.clone()))
                }
                _ => None,
            })
            .collect()
    }
    fn update_failed_worker_state(&mut self, worker_id: WorkerId) {
        todo!();
    }
    fn handle_worker_failure(&mut self, worker_id: WorkerId) {
        self.update_failed_worker_state(worker_id);
    }
}

impl ComponentLifecycle for Master {
    fn on_start(&mut self) -> Handled {
        for i in 0..self.worker_states.len() {
            let worker = self
                .ctx
                .system()
                .create(|| Worker::new(i.try_into().unwrap()));

            worker.connect_to_required(self.message_port.share());

            let worker_ref = worker.actor_ref().hold().expect("hold the worker refs");
            self.ctx.system().start(&worker);
            self.worker_components.push(worker);
            self.worker_refs.push(worker_ref);
        }
        self.request_for_proposal();
        Handled::Ok
    }
    fn on_stop(&mut self) -> Handled {
        self.worker_refs.clear();
        let sys = self.ctx.system();
        self.worker_components.drain(..).for_each(|worker| {
            sys.stop(&worker);
        });
        Handled::Ok
    }
    fn on_kill(&mut self) -> Handled {
        self.on_stop()
    }
}

impl Actor for Master {
    type Message = WorkerResponse;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        Handled::Ok
    }
    fn receive_network(&mut self, _msg: NetMessage) -> Handled {
        unimplemented!("Not written for Network Messages currently")
    }
}

#[derive(Debug, Clone)]
pub enum MasterMessage {
    Rfp {
        master_clock: LamportClock,
    },
    AcceptedProposalBroadcast {
        logical_time: LamportClock,
        broadcast_message: BroadcastMessage,
    },
}

pub struct MessagePort;

impl Port for MessagePort {
    type Indication = WorkerResponse;
    type Request = MasterMessage;
}

impl Require<MessagePort> for Master {
    fn handle(&mut self, event: WorkerResponse) -> Handled {
        match event {
            WorkerResponse::RfpResponse(ref msg) => {
                println!("event {:?}", msg);
                self.worker_response
                    .push(Some(WorkerResponse::RfpResponse(msg.clone())));

                // NOTE: below doesn't manage bugged worker states, will fix later

                if self.worker_response.len() == self.worker_states.len() {
                    info!(self.ctx.log(), "proposals received from all workers");
                    println!("proposals received from all workers");
                    self.process_proposals();
                }
            }
            WorkerResponse::StateUpdateConfirmed {
                worker_id,
                logical_time,
            } => {
                if let Some(worker_state) = self.worker_states.get_mut(&WorkerId(worker_id)) {
                    *worker_state =
                        WorkerState::Active(ActiveWorkerStates::ProcessingQueuedMessages);
                }

                info!(
                    self.ctx.log(),
                    "state update confirmed for worker {}", worker_id
                );
            }
            _ => debug!(
                self.ctx.log(),
                "Error: WorkerResponse::NoResponse sent to master"
            ),
        }
        Handled::Ok
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WorkerId(pub u8);

//TODO: Continue developing logic for master acknowledgement of workerstate, ActiveWorkerState
//necessary to delineate between a worker with an empty queue and a faulty/dead worker. Add logic
// in rfp response handling for consideration of worker_state, and add logic for updating master's
// internal storage of worker state based on worker_response to rfp.
// i.e. if emtpy_queue on worker -> worker response should call method on WorkerState (inside match
// from master) that updates to EmptyMessageQueue, master should no longer expect rfp to generate a proposal
#[derive(Debug, Clone, Default, Hash, PartialEq)]
pub enum WorkerState {
    #[default]
    Start,
    Active(ActiveWorkerStates),
    PossibleFault,
    Dead,
}

impl WorkerState {
    fn active(&mut self) -> Self {
        match self {
            WorkerState::Active(state) => WorkerState::Active(state.handle_active_state()),
            _ => todo!(),
        }
    }
    fn possible_fault(&mut self) -> Self {
        WorkerState::PossibleFault
    }

    fn dead_worker_shutdown(&mut self) -> Self {
        WorkerState::Dead
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ActiveWorkerStates {
    ProcessingQueuedMessages,
    EmptyMessageQueue,
}

impl ActiveWorkerStates {
    fn handle_active_state(&mut self) -> Self {
        match self {
            ActiveWorkerStates::ProcessingQueuedMessages => todo!(),
            ActiveWorkerStates::EmptyMessageQueue => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialOrd, PartialEq)]
pub struct LamportClock {
    pub time: u64,
}

impl LamportClock {
    pub fn new() -> Self {
        LamportClock { time: 0 }
    }
    pub fn increment(&mut self) {
        self.time += 1;
    }
    pub fn adjust(&mut self, incoming_time: u64) {
        self.time = std::cmp::max(self.time, incoming_time) + 1;
    }
    pub fn time(&self) -> u64 {
        self.time
    }
}

impl Ord for LamportClock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}
