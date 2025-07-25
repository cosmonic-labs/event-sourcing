/// Generated WIT bindings
mod bindings {
    use super::EventSourcer;
    wit_bindgen::generate!({
        path: "../wit",
        world: "event-sourcer-w",
    });

    export!(EventSourcer);
}

use bindings::cosmonic::eventsourcing::*;
use bindings::exports::cosmonic::eventsourcing::*;

struct EventSourcer;

impl event_sourcer::Guest for EventSourcer {
    fn get_events(aggregate_id: String) -> Result<Vec<types::Event>, String> {
        let bytes = event_store::get_events(&aggregate_id)?;
        let mut events = Vec::with_capacity(bytes.len());
        for event_bytes in bytes {
            events.push(
                aggregate::deserialize_event(&event_bytes)
                    .map_err(|e| format!("failed to deserialize evente: {e}"))?,
            );
        }

        Ok(events)
    }

    fn append(aggregate_id: String, new_events: Vec<types::Event>) -> Result<Vec<Vec<u8>>, String> {
        let mut all_events = Vec::with_capacity(new_events.len());

        for event in new_events {
            let event_bytes = aggregate::serialize_event(event)
                .map_err(|e| format!("Failed to serialize event: {e}"))?;
            event_store::append_event(&aggregate_id, &event_bytes)?;
            all_events.push(event_bytes);
        }

        Ok(all_events)
    }

    fn handle_command(
        aggregate_id: String,
        command: Vec<u8>,
    ) -> Result<Vec<event_sourcer::Event>, String> {
        let events_bytes = event_store::get_events(&aggregate_id)?;
        let mut events = Vec::with_capacity(events_bytes.len());

        for event in events_bytes {
            events.push(aggregate::deserialize_event(&event)?);
        }
        let state = aggregate::rehydrate(events)?;
        let command = aggregate::deserialize_command(&command)?;

        let events = aggregate::handle(state, command)?;

        Ok(events)
    }
}
