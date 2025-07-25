fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the proto file directly from aggregate
    let proto_file = "./aggregate/proto/bank.proto";

    prost_build::compile_protos(&[proto_file], &["./aggregate/proto/"])?;
    Ok(())
}
