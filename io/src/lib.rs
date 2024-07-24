#![no_std]

use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId, CodeId};

pub type Id = u64;

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FactoryAction {
    CreateProgram { init_config: InitConfig },
    CodeIdUpdate { new_code_id: CodeId },
    UpdateGasProgram(u64),
    AddAdmin { admin_actor_id: ActorId },
    RemoveRegistry { id: Id },
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Record {
    pub field: String,
   
}

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FactoryEvent {
    ProgramCreated {
        id: Id,
        address: ActorId,
        init_config: InitConfig,
    },
    GasUpdatedSuccessfully {
        updated_by: ActorId,
        new_gas_amount: u64,
    },
    CodeIdUpdatedSuccessfully {
        updated_by: ActorId,
        new_code_id: CodeId,
    },
    AdminAdded {
        updated_by: ActorId,
        admin_actor_id: ActorId,
    },
    RegistryRemoved {
        removed_by: ActorId,
        program_for_id: Id,
    },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FactoryError {
    ProgramInitializationFailed,
    ProgramInitializationFailedWithContext(String),
    Unauthorized,
    UnexpectedFTEvent,
    MessageSendError,
    NotFound,
    IdNotFoundInAddress,
    IdNotFound
}


#[derive(Debug, Decode, Encode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitConfig {
    pub field: String,
    
}


#[derive(Debug, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitConfigFactory {
    pub code_id: CodeId,
    pub factory_admin_account: Vec<ActorId>,
    pub gas_for_program: u64,
}

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = In<InitConfigFactory>;
    type Handle = InOut<FactoryAction, Result<FactoryEvent, FactoryError>>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<Query, QueryReply>;
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Query {
    Number,
    CodeId,
    FactoryAdminAccount,
    GasForProgram,
    IdToAddress,
    Registry
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum QueryReply {
    Number(Id),
    CodeId(CodeId),
    FactoryAdminAccount(Vec<ActorId>),
    GasForProgram(u64),
    IdToAddress(Vec<(Id, ActorId)>),
    Registry(Vec<(ActorId, Vec<(Id, Record)>)>)
}
