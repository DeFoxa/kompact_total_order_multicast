use crate::worker_types::*;
use kompact::prelude::*;

#[derive(ComponentDefinition)]
pub struct Master {
    ctx: ComponentContext<Self>,
}

impl Master {
    fn new() -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
        }
    }
}
impl ComponentLifecycle for Master {
    fn on_start(&mut self) -> Handled {
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
pub enum MasterRequest {
    Rfp,
    SequenceNumber { seq_number: i32, message: i32 },
}

pub struct MessagePort;

impl Port for MessagePort {
    type Indication = WorkerResponse;
    type Request = MasterRequest;
}
