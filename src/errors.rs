///NOTE: complete error state development and integrate into master -> worker communication logic
pub enum MessageError {
    MasterInternalError,
    WorkerInternalError,
    MessagePassingError,
}

pub enum MasterInternalError {
    ProposalError(ProposalHandlingErrors),
}

pub enum ProposalHandlingErrors {
    EmptyQueue,
    ProposalProcessingError,
    ProposalFilteringError,
}
impl ProposalHandlingErrors {}
pub enum MessagePassingError {
    BroadcastError,
    ResponseError,
}
pub enum BroadcastError {
    UnresponsiveWorker { worker_id: u8 },
    UnresponsiveMaster,
}
pub enum ResponseError {
    MissedRfp,
    MissedStateUpdateConfirmation,
}
