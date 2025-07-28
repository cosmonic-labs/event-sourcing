fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the proto file directly from command_handler
    let proto_file = "./command_handler/proto/bank.proto";

    prost_build::compile_protos(&[proto_file], &["./command_handler/proto/"])?;
    Ok(())
}
