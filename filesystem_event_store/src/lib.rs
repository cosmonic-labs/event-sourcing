use uuid::{Context, Timestamp, Uuid};
use wasi::filesystem::{
    preopens::get_directories,
    types::{DescriptorFlags, DescriptorType, OpenFlags, PathFlags},
};

/// Generated WIT bindings
mod bindings {
    use super::EventStore;
    wit_bindgen::generate!({
        path: "../wit",
        world: "event-store-w",
    });

    export!(EventStore);
}

use bindings::exports::cosmonic::eventsourcing::*;

struct EventStore;

impl event_store::Guest for EventStore {
    fn get_events(command_handler_id: String) -> Result<Vec<Vec<u8>>, String> {
        let dirs = get_directories();
        if dirs.is_empty() {
            return Err("No root directory available for persistence".to_string());
        }

        let event_root = &dirs[0].0;
        let command_handler_state_dir = event_root
            .open_at(
                PathFlags::empty(),
                &command_handler_id,
                // TODO: create if not exist seems to fail. When we get this we need to check
                // for existence first, then create if not exist
                OpenFlags::DIRECTORY,
                DescriptorFlags::empty(),
            )
            .map_err(|e| format!("Failed to open command_handler folder: {e:?}"))?;

        let dir_stream = command_handler_state_dir
            .read_directory()
            .map_err(|e| format!("Failed to read command_handler state: {e:?}"))?;
        let mut events = vec![];
        while let Ok(Some(dir)) = dir_stream.read_directory_entry() {
            if dir.type_ == DescriptorType::RegularFile {
                let event_file = command_handler_state_dir
                    .open_at(
                        PathFlags::empty(),
                        &dir.name,
                        OpenFlags::empty(),
                        DescriptorFlags::READ,
                    )
                    .map_err(|e| format!("Failed to read command_handler data: {e:?}"))?;
                let stat = event_file
                    .stat()
                    .map_err(|e| format!("Failed to get descriptor information: {e:?}"))?;
                let (bytes, _read_all) = event_file
                    .read(stat.size, 0)
                    .map_err(|e| format!("Failed to read command_handler data: {e:?}"))?;

                events.push(bytes);
            }
        }

        Ok(events)
    }

    fn append_event(command_handler_id: String, event: Vec<u8>) -> Result<(), String> {
        let dirs = get_directories();
        if dirs.is_empty() {
            return Err("No root directory available for persistence".to_string());
        }

        let event_root = &dirs[0].0;
        let command_handler_state_dir = event_root
            .open_at(
                PathFlags::empty(),
                &command_handler_id,
                OpenFlags::DIRECTORY,
                DescriptorFlags::MUTATE_DIRECTORY,
            )
            .map_err(|e| format!("Failed to open command_handler folder: {e:?}"))?;

        // A v7 uuid ensures events sorted by timestamp and is the "true" event time.
        // let ts = wasi::clocks::wall_clock::now();
        // let random = wasi::random::random::get_random_u64();
        // let context = ContextV7::from(random);
        // let ts = uuid::timestamp::Timestamp::from_unix(NoContext, ts.seconds, ts.nanoseconds);
        // let ts = Timestamp::from_gregorian(0, 0);
        // TODO: random context, current time
        let random_u64 = wasi::random::insecure::get_insecure_random_u64();
        let random_u16 = (random_u64 & 0xFFFF) as u16;
        let context = Context::new(random_u16);
        let date_time = wasi::clocks::wall_clock::now();
        let ts = Timestamp::from_unix(context, date_time.seconds, date_time.nanoseconds);
        let event_id = Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);

        let event_file = command_handler_state_dir
            .open_at(
                PathFlags::empty(),
                &event_id.to_string(),
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            )
            .map_err(|e| format!("failed to open event file: {e:?}"))?;
        event_file
            .write(&event, 0)
            .map_err(|e| format!("Failed to write command_handler data: {e:?}"))?;

        Ok(())
    }
}
