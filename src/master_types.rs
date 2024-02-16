use crate::worker_types::*;
use anyhow::Result;
use futures::future::join;
use kompact::prelude::*;
use rand::Rng;
use std::sync::Arc;
use std::{thread, time};

#[derive(ComponentDefinition)]
pub struct Master {
    ctx: ComponentContext<Self>,
    message_port: RequiredPort<MessagePort>,
    worker_count: u8,
    workers: Vec<Arc<Component<Worker>>>,
    // worker_response: Vec<WorkerResponse>,
    outstanding_proposals: Option<Ask<MasterMessage, WorkerResponse>>,
    worker_response: Vec<Option<WorkerResponse>>,
    worker_refs: Vec<ActorRefStrong<WorkerMessages>>,
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
        }
    }
    fn request_for_proposal(&mut self) {
        //adding jitter component to simulate network latency for communication between master and
        //workers
        let mut rng = rand::thread_rng();
        let jitter: u32 = rng.gen_range(25..100);
        let delay_duration = std::time::Duration::from_millis((jitter).into());
        let workers = self.worker_refs.clone();

        for _ in workers.iter() {
            self.schedule_once(delay_duration, move |new_self, _context| {
                new_self.message_port.trigger(MasterMessage::Rfp);
                new_self.ctx().system().shutdown_async();
                Handled::Ok
            });
        }
    }
    async fn process_response(&mut self) {
        todo!();
    }
    fn process_proposals(&self) {
        todo!();
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
    Rfp,
    AcceptedProposalBroadcast { seq_number: i32, message: u8 },
}

pub struct MessagePort;

impl Port for MessagePort {
    type Indication = WorkerResponse;
    type Request = MasterMessage;
}

impl Require<MessagePort> for Master {
    fn handle(&mut self, event: WorkerResponse) -> Handled {
        match event {
            WorkerResponse::RfpResponse(event) => {
                println!("event {:?}", event);
                self.worker_response
                    .push(Some(WorkerResponse::RfpResponse(event)));
                if self.worker_response.len() == self.worker_count.into() {
                    info!(self.ctx.log(), "proposals received from all workers");
                    println!("proposals received from all workers");
                    self.process_proposals();
                }
            }
            WorkerResponse::StateUpdateConfirmed => {
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
