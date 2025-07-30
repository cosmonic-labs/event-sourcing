/// Generated WIT bindings
pub mod bindings;
pub mod types;
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

use bindings::exports::cosmonic::eventsourcing::{
    command_handler,
    types::{Command, Event, State},
};
use proto::{
    bank_command, bank_event, AccountOpened, BankCommand, BankEvent, BankState, Transaction,
    TransactionDenied,
};

pub struct CommandHandler;

impl command_handler::Guest for CommandHandler {
    fn rehydrate(events: Vec<Event>) -> Result<State, String> {
        let state = events
            .iter()
            .fold(BankState::default(), |mut state, event| {
                match event.get::<BankEvent>().event.as_ref() {
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
                state
            });

        Ok(state.into())
    }

    fn handle(state: State, command: Command) -> Result<Vec<Event>, String> {
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
}
