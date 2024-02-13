use crate::worker_types::*;
use kompact::prelude::*;

#[derive(ComponentDefinition)]
pub struct Master {
    ctx: ComponentContext<Self>,
    message_port: RequiredPort<MessagePort>,
    worker_count: usize,
    // worker_response: Vec<WorkerResponse>,
    outstanding_proposals: Option<Ask<MasterMessage, WorkerResponse>>,
    worker_refs: Vec<ActorRefStrong<WorkerInternalMessage>>,
}

impl Master {
    fn new(num_workers: usize) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            message_port: RequiredPort::uninitialised(),
            worker_count: num_workers,
            // worker_response: Vec::new(),
            outstanding_proposals: None,
            worker_refs: Vec::with_capacity(num_workers),
        }
    }
    pub fn request_for_proposal(&self) {
        let msg = MasterMessage::MasterRequest::Rfp;
    }
    pub fn broadcast_message_to_workers(&self, message: MasterMessage) {
        for worker in &self.worker_refs {
            worker.tell(MasterMessage::MasterRequest(message.clone()));
        }
    }
}
impl ComponentLifecycle for Master {
    fn on_start(&mut self) -> Handled {
        for _ in 0..self.worker_count {
            let worker = self.ctx.system().create(|| Worker::new());
            let worker_ref = worker.actor_ref().hold().expect("hold the worker refs");
            self.worker_refs.push(worker_ref);
            self.ctx.system().start(&worker);
        }
        Handled::Ok
    }
    fn on_stop(&mut self) -> Handled {
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
    SequenceNumber { seq_number: i32, message: i32 },
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
