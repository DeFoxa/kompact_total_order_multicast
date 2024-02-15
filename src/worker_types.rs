use crate::master_types::*;
use kompact::prelude::*;

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
    proposed_sequence_number: Option<i32>,
    msg: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct StateUpdate {
    seq_number: i32,
    message: i32,
}

#[derive(ComponentDefinition)]
pub struct Worker {
    ctx: ComponentContext<Self>,
    state: (i32, i32),
    proposed_sequence_number: Option<i32>,
    message_port: ProvidedPort<MessagePort>,
}
ignore_lifecycle!(Worker);

impl Worker {
    pub fn new() -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            state: (0, 0),
            proposed_sequence_number: None,
            message_port: ProvidedPort::uninitialised(),
        }
    }

    fn update_state(&mut self, seq_number: i32, message: i32) {
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

    fn handle_accepted_proposal(&mut self) -> WorkerResponse {
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
            } => self.handle_accepted_proposal(),
        }
    }
}

impl Actor for Worker {
    type Message = WorkerMessages;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        match msg {
            WorkerMessages::External(master_request) => self.handle_external(master_request),
            WorkerMessages::InternalStateUpdate(state_update) => {
                self.update_state(state_update.seq_number, state_update.message);
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
