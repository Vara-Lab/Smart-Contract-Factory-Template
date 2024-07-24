#![no_std]
use gstd::{
    async_main, collections::HashMap, msg, prelude::*, prog::ProgramGenerator, ActorId, CodeId,
};
use io::*;

#[cfg(feature = "binary-vendor")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

static mut FACTORY: Option<StateFactory> = None;


#[derive(Debug, Default)]
pub struct StateFactory {
    pub number: Id,
    pub code_id: CodeId,
    pub factory_admin_account: Vec<ActorId>,
    pub gas_for_program: u64,
    pub id_to_address: HashMap<Id, ActorId>,
    pub registry: HashMap<ActorId, Vec<(Id, Record)>>,
}

impl StateFactory {
    pub async fn create_program(
        &mut self,
        init_config: InitConfig,
    ) -> Result<FactoryEvent, FactoryError> {
        
        let create_program_future =
            ProgramGenerator::create_program_with_gas_for_reply::<InitConfig>(
                self.code_id,
                InitConfig {
                    field: init_config.field.clone(),
                    
                },
                self.gas_for_program,
                0,
                10_000_000_000,
            )
            .map_err(|e| FactoryError::ProgramInitializationFailedWithContext(e.to_string()))?;

        let (address, _) = create_program_future
            .await
            .map_err(|e| FactoryError::ProgramInitializationFailedWithContext(e.to_string()))?;

        self.number = self.number.saturating_add(1);

        self.id_to_address
            .entry(self.number)
            .or_insert(address);

        let record = Record {
            field: init_config.field.clone(),
        };

        let programs_for_actor = self.registry.entry(msg::source()).or_default();
        programs_for_actor.push((self.number, record.clone()));

        Ok(FactoryEvent::ProgramCreated {
            id: self.number,
            address: address,
            init_config: init_config,
        })
    }

    pub fn update_gas_for_program(
        &mut self,
        new_gas_amount: u64,
    ) -> Result<FactoryEvent, FactoryError> {
        if self.factory_admin_account.contains(&msg::source()) {
            self.gas_for_program = new_gas_amount;
            Ok(FactoryEvent::GasUpdatedSuccessfully {
                updated_by: msg::source(),
                new_gas_amount,
            })
        } else {
            return Err(FactoryError::Unauthorized);
        }
    }

    pub fn update_code_id(&mut self, new_code_id: CodeId) -> Result<FactoryEvent, FactoryError> {
        if self.factory_admin_account.contains(&msg::source()) {
            self.code_id = new_code_id;
            Ok(FactoryEvent::CodeIdUpdatedSuccessfully {
                updated_by: msg::source(),
                new_code_id,
            })
        } else {
            return Err(FactoryError::Unauthorized);
        }
    }

    pub fn add_admin_to_factory(
        &mut self,
        admin_actor_id: ActorId,
    ) -> Result<FactoryEvent, FactoryError> {
        if self.factory_admin_account.contains(&msg::source()) {
            self.factory_admin_account.push(admin_actor_id);

            Ok(FactoryEvent::AdminAdded {
                updated_by: msg::source(),
                admin_actor_id,
            })
        } else {
            return Err(FactoryError::Unauthorized);
        }
    }

    pub fn remove_registry(&mut self, program_for_id: Id) -> Result<FactoryEvent, FactoryError> {
        let source = msg::source();
        if self.factory_admin_account.contains(&source) {
            if self.id_to_address.remove(&program_for_id).is_none() {
                return Err(FactoryError::IdNotFoundInAddress);
            }

            let mut is_removed = false;

            for (_actor_id, info) in self.registry.iter_mut() {
                if let Some(pos) = info.iter().position(|(id, _)| *id == program_for_id) {
                    info.remove(pos);
                    is_removed = true;
                    break;
                }
            }

            if !is_removed {
                return Err(FactoryError::IdNotFound);
            }

            Ok(FactoryEvent::RegistryRemoved  {
                removed_by: source,
                program_for_id,
            })
        } else {
            return Err(FactoryError::Unauthorized);
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    let init_config_factory: InitConfigFactory =
        msg::load().expect("Unable to decode CodeId of the program");

    let factory = StateFactory {
        number: 0,
        code_id: init_config_factory.code_id,
        factory_admin_account: init_config_factory.factory_admin_account,
        gas_for_program: init_config_factory.gas_for_program,
        ..Default::default()
    };
    unsafe { FACTORY = Some(factory) };
}

#[async_main]
async fn main() {
    let action: FactoryAction = msg::load().expect("Could not load Action");

    let factory_state = unsafe {
        FACTORY
            .as_mut()
            .expect("Unexpected error in factory_state")
    };

    let result = match action {
        FactoryAction::CreateProgram { init_config } => {
            factory_state.create_program(init_config).await
        }
        FactoryAction::AddAdmin { admin_actor_id } => {
            factory_state.add_admin_to_factory(admin_actor_id)
        }
        FactoryAction::UpdateGasProgram(new_gas_amount) => {
            factory_state.update_gas_for_program(new_gas_amount)
        }
        FactoryAction::CodeIdUpdate { new_code_id } => {
            factory_state.update_code_id(new_code_id)
        }
        FactoryAction::RemoveRegistry  { id } => factory_state.remove_registry(id),
    };

    msg::reply(result, 0)
        .expect("Failed to encode or reply with `Result<FactoryEvent, FactoryError>`");
}

#[no_mangle]
extern "C" fn state() {
    let factory_state = unsafe {
        FACTORY
            .take()
            .expect("Unexpected error in taking state")
    };
    let query: Query = msg::load().expect("Unable to decode the query");
    let reply = match query {
        Query::Number => QueryReply::Number(factory_state.number),
        Query::CodeId => QueryReply::CodeId(factory_state.code_id),
        Query::FactoryAdminAccount => {
            QueryReply::FactoryAdminAccount(factory_state.factory_admin_account)
        }
        Query::GasForProgram => QueryReply::GasForProgram(factory_state.gas_for_program),
        Query::IdToAddress => {
            QueryReply::IdToAddress(factory_state.id_to_address.into_iter().collect())
        }

        Query::Registry => QueryReply::Registry(factory_state.registry.into_iter().collect()),
    };
    msg::reply(reply, 0).expect("Error on state");
}
