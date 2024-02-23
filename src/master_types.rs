use crate::worker_types::*;
use anyhow::{anyhow, Result};
use futures::future::join;
use kompact::prelude::*;
use rand::Rng;
use std::sync::Arc;
use std::{thread, time};

//TODO: write method to process proposals from workers to determined accepted proposal
// why do i have a process_response and process_proposal method? should only need one unless im
// going to something specific with the StateUpdateConfirmed response. Also do I still need
// outstanding_proposal field? Figure out what I'm doing with the receive_local on master, if
// anything. may do all responses through port
//
//
// TODO: Longer term todo: write worker and master handling for tracking active workers on master
// side, so master has a way to verify active and inactive workers and handle state update
// confirmation and outstanding rfp

#[derive(ComponentDefinition)]
pub struct Master {
    ctx: ComponentContext<Self>,
    message_port: RequiredPort<MessagePort>,
    worker_count: u8,
    workers: Vec<Arc<Component<Worker>>>,
    // worker_response: Vec<WorkerResponse>,
    outstanding_proposals: Option<Ask<MasterMessage, WorkerResponse>>,
    worker_response: Vec<Option<WorkerResponse>>,
    worker_refs: Vec<ActorRefStrong<MasterMessage>>,
    local_clock: LamportClock,
}

impl Master {
    fn new(num_workers: u8) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            message_port: RequiredPort::uninitialised(),
            worker_count: num_workers,
            workers: Vec::with_capacity(num_workers.into()),
            // worker_response: Vec::new(),
            outstanding_proposals: None,
            worker_response: Vec::with_capacity(num_workers.into()),
            worker_refs: Vec::with_capacity(num_workers.into()),
            local_clock: LamportClock::new(),
        }
    }
    fn request_for_proposal(&mut self) {
        //adding jitter component to simulate network latency for communication between master and
        //workers, also introduces possiblity of asynchronous rfp/accepted receipt by worker which
        //should be handled by btree and lamport clock
        let mut rng = rand::thread_rng();
        let jitter: u32 = rng.gen_range(25..100);
        let delay_duration = std::time::Duration::from_millis((jitter).into());
        let workers = self.worker_refs.clone();

        for _ in workers.iter() {
            self.schedule_once(delay_duration, move |new_self, _context| {
                new_self.message_port.trigger(MasterMessage::Rfp {
                    master_clock: new_self.local_clock,
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
                    seq_number: broadcast_message.sequence_number,
                    message: broadcast_message.content,
                    logical_time: lamport_clock.clone(),
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
}
impl ComponentLifecycle for Master {
    fn on_start(&mut self) -> Handled {
        for i in 0..self.worker_count {
            let worker = self.ctx.system().create(|| Worker::new(i));
            worker.connect_to_required(self.message_port.share());
            let worker_ref = worker.actor_ref().hold().expect("hold the worker refs");
            self.ctx.system().start(&worker);
            self.workers.push(worker);
            self.worker_refs.push(worker_ref);
        }
        self.request_for_proposal();
        Handled::Ok
    }
    fn on_stop(&mut self) -> Handled {
        self.worker_refs.clear();
        let sys = self.ctx.system();
        self.workers.drain(..).for_each(|worker| {
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
        seq_number: i64,
        message: u8,
        logical_time: LamportClock,
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

                // NOTE: below doesn't manage bugged worker states, will fix later:
                // if single worker responds multiple times or doesn't respond,
                // system won't run proposal comparisons

                if self.worker_response.len() == self.worker_count.into() {
                    info!(self.ctx.log(), "proposals received from all workers");
                    println!("proposals received from all workers");
                    self.process_proposals();
                }
            }
            WorkerResponse::StateUpdateConfirmed {
                worker_id,
                logical_time,
            } => {
                todo!(); //internally acknowledge response
            }
            _ => debug!(
                self.ctx.log(),
                "Error: WorkerResponse::NoResponse sent to master"
            ),
        }

        todo!();
    }
}
#[derive(Debug, Clone, Copy)]
pub struct LamportClock {
    time: u64,
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
