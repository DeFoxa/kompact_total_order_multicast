use crate::master_types::*;
use kompact::prelude::*;

#[derive(Debug, Clone)]
pub struct WorkerResponse {
    proposed_sequence_number: Option<i32>,
    msg: Option<i32>,
}

#[derive(Debug)]
pub struct StateUpdate {
    seq_number: i32,
    message: i32,
}

#[derive(ComponentDefinition)]
pub struct Worker {
    ctx: ComponentContext<Self>,
    state: (i32, i32),
    proposed_sequence_number: i32,
    message_port: ProvidedPort<MessagePort>,
}
ignore_lifecycle!(Worker);

impl Worker {
    fn update_state(&mut self, seq_number: i32, message: i32) {
        todo!();
    }
}

impl Actor for Worker {
    type Message = StateUpdate;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        self.update_state(msg.seq_number, msg.message);

        Handled::Ok
    }
    fn receive_network(&mut self, _msg: NetMessage) -> Handled {
        unimplemented!("No receive network message handling on Worker")
    }
}

impl Provide<MessagePort> for Worker {
    fn handle(&mut self, event: MasterRequest) -> Handled {
        match event {
            MasterRequest::Rfp => {
                //generate and return proposal response to Rfp Req
                todo!();
            }
            MasterRequest::SequenceNumber {
                seq_number,
                message,
            } => {
                self.actor_ref().tell(StateUpdate {
                    seq_number,
                    message,
                });
            }
        };
        Handled::Ok
    }
}

//
// #[derive(Debug)]
// pub enum InternalCommand {
//     UpdateState { seq_num: i32, message: i32 },
//     Start,
//     Stop,
//     Kill,
// }
// //Response status is for handling worker internal state prior to receivng/sending responses to Master
// #[derive(Debug, Clone)]
// enum ResponseStatus {
//     Proposal,
//     Acknowledged,
// }
//
// fn default_response() -> WorkerResponse {
//     WorkerResponse {
//         proposed_sequence_number: None,
//         msg: None,
//         status: ResponseStatus::Acknowledged,
//     }
// }
