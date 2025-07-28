/// Generated WIT bindings
mod bindings {
    use super::CommandHandler;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "command-handler-w",
    });

    export!(CommandHandler);
}

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

use prost::Message;
use proto::{
    bank_command, bank_event, AccountOpened, BankCommand, BankEvent, BankState, Transaction,
    TransactionDenied,
};

impl From<BankEvent> for types::Event {
    fn from(value: BankEvent) -> Self {
        types::Event::new(value)
    }
}
impl From<bank_event::Event> for types::Event {
    fn from(value: bank_event::Event) -> Self {
        types::Event::new(BankEvent { event: Some(value) })
    }
}
impl From<BankCommand> for types::Command {
    fn from(value: BankCommand) -> Self {
        types::Command::new(value)
    }
}
impl From<BankState> for types::State {
    fn from(value: BankState) -> Self {
        types::State::new(value)
    }
}

pub struct CommandHandler;

use bindings::exports::cosmonic::eventsourcing::*;

impl types::GuestEvent for BankEvent {}
impl types::GuestCommand for BankCommand {}
impl types::GuestState for BankState {}
impl types::Guest for CommandHandler {
    type Event = BankEvent;
    type Command = BankCommand;
    type State = BankState;
}

impl command_handler::Guest for CommandHandler {
    fn rehydrate(events: Vec<types::Event>) -> Result<types::State, String> {
        let mut state = BankState::default();
        for e in events {
            match e.get::<BankEvent>().event.as_ref() {
                Some(bank_event::Event::Opened(AccountOpened { balance, id })) => {
                    state.balance = i64::from(*balance);
                    state.id = id.to_owned();
                    state.is_open = true;
                }
                Some(bank_event::Event::Transaction(Transaction { amount })) => {
                    state.balance += amount;
                }
                Some(bank_event::Event::Denied(TransactionDenied { amount: _ })) => {
                    // could add a denied log to the account, or something
                }
                None => {}
            }
        }
        Ok(state.into())
    }

    fn handle(state: types::State, command: types::Command) -> Result<Vec<types::Event>, String> {
        let bank_state: &BankState = state.get();
        match command.into_inner::<BankCommand>().command {
            Some(bank_command::Command::Transaction(amount)) => {
                if bank_state.balance.saturating_sub(amount) < 0 {
                    Ok(vec![bank_event::Event::Denied(TransactionDenied {
                        amount,
                    })
                    .into()])
                } else {
                    Ok(vec![
                        bank_event::Event::Transaction(Transaction { amount }).into()
                    ])
                }
            }
            Some(bank_command::Command::OpenAccount(initial_balance)) => {
                if bank_state.is_open {
                    return Err("Account is already open".to_string());
                }
                Ok(vec![bank_event::Event::Opened(AccountOpened {
                    balance: initial_balance,
                    id: uuid::Uuid::now_v7().to_string(),
                })
                .into()])
            }
            None => Ok(vec![]),
        }
    }

    fn serialize_event(event: types::Event) -> Result<Vec<u8>, String> {
        Ok(event.into_inner::<BankEvent>().encode_to_vec())
    }

    fn deserialize_event(event: Vec<u8>) -> Result<types::Event, String> {
        Ok(BankEvent::decode(event.as_slice())
            .map_err(|e| format!("Event deserialization failed: {e}"))?
            .into())
    }

    fn serialize_command(command: types::Command) -> Result<Vec<u8>, String> {
        Ok(command.into_inner::<BankCommand>().encode_to_vec())
    }

    fn deserialize_command(command: Vec<u8>) -> Result<types::Command, String> {
        Ok(BankCommand::decode(command.as_slice())
            .map_err(|e| format!("Command deserialization failed: {e}"))?
            .into())
    }

    fn serialize_state(state: types::State) -> Result<Vec<u8>, String> {
        let bank_state = state.into_inner::<BankState>();
        let proto_state = proto::BankState {
            balance: bank_state.balance,
            id: bank_state.id,
            is_open: bank_state.is_open,
        };
        Ok(proto_state.encode_to_vec())
    }

    fn deserialize_state(state: Vec<u8>) -> Result<types::State, String> {
        let proto_state = proto::BankState::decode(state.as_slice())
            .map_err(|e| format!("State deserialization failed: {e}"))?;
        let bank_state = BankState {
            balance: proto_state.balance,
            id: proto_state.id,
            is_open: proto_state.is_open,
        };
        Ok(types::State::new(bank_state))
    }
}
