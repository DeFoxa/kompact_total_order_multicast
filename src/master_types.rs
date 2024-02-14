use crate::worker_types::*;
use kompact::prelude::*;
use std::sync::Arc;

#[derive(ComponentDefinition)]
pub struct Master {
    ctx: ComponentContext<Self>,
    message_port: RequiredPort<MessagePort>,
    worker_count: usize,
    workers: Vec<Arc<Component<Worker>>>,
    // worker_response: Vec<WorkerResponse>,
    outstanding_proposals: Option<Ask<MasterMessage, WorkerResponse>>,
    worker_response: Vec<Option<External>>,
    worker_refs: Vec<ActorRefStrong<WorkerMessages>>,
}

impl Master {
    fn new(num_workers: usize) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            message_port: RequiredPort::uninitialised(),
            worker_count: num_workers,
            workers: Vec::with_capacity(num_workers),
            // worker_response: Vec::new(),
            outstanding_proposals: None,
            worker_response: Vec::with_capacity(num_workers),
            worker_refs: Vec::with_capacity(num_workers),
        }
    }
    fn request_for_proposal(&mut self, req: MasterMessage) {
        let msg = MasterMessage::Rfp;
        if self.outstanding_proposals.is_some() {
            let ask = self.outstanding_proposals.take().expect("ask");
            let rfp = ask.request();
            //TODO: handle sending ask and response
        }
        for worker in &self.worker_refs {}
    }
    fn broadcast_accepted_proposal(&self, message: MasterMessage) {
        for worker in &self.worker_refs {
            worker.tell(message.clone());
        }
    }
}
impl ComponentLifecycle for Master {
    fn on_start(&mut self) -> Handled {
        for _ in 0..self.worker_count {
            let worker = self.ctx.system().create(|| Worker::new());
            worker.connect_to_required(self.message_port.share());
            let worker_ref = worker.actor_ref().hold().expect("hold the worker refs");
            self.ctx.system().start(&worker);
            self.workers.push(worker);
            self.worker_refs.push(worker_ref);
        }
        self.request_for_proposal(MasterMessage::Rfp);
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
    AcceptedProposalBroadcast { seq_number: i32, message: i32 },
}

pub struct MessagePort;

impl Port for MessagePort {
    type Indication = WorkerResponse;
    type Request = MasterMessage;
}

impl Require<MessagePort> for Master {
    fn handle(&mut self, event: WorkerResponse) -> Handled {
        todo!();
    }
}
