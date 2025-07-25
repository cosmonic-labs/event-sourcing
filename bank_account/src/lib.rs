pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

pub use proto::*;
