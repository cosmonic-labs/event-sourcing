use prost::Message as _;

use crate::bindings::exports::cosmonic::eventsourcing::*;
use crate::proto::*;

impl types::Guest for crate::CommandHandler {
    type Event = BankEvent;
    type Command = BankCommand;
    type State = BankState;
}
impl types::GuestEvent for BankEvent {
    fn serialize(&self) -> Result<Vec<u8>, String> {
        Ok(self.encode_to_vec())
    }
    fn deserialize(event: Vec<u8>) -> Result<types::Event, String> {
        Ok(BankEvent::decode(event.as_slice())
            .map_err(|e| format!("Event deserialization failed: {e}"))?
            .into())
    }
}
impl types::GuestCommand for BankCommand {
    fn serialize(&self) -> Result<Vec<u8>, String> {
        Ok(self.encode_to_vec())
    }
    fn deserialize(command: Vec<u8>) -> Result<types::Command, String> {
        Ok(BankCommand::decode(command.as_slice())
            .map_err(|e| format!("Event deserialization failed: {e}"))?
            .into())
    }
}
impl types::GuestState for BankState {
    fn serialize(&self) -> Result<Vec<u8>, String> {
        Ok(self.encode_to_vec())
    }
    fn deserialize(state: Vec<u8>) -> Result<types::State, String> {
        Ok(BankState::decode(state.as_slice())
            .map_err(|e| format!("Event deserialization failed: {e}"))?
            .into())
    }
}

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
