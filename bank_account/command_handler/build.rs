fn main() {
    prost_build::compile_protos(&["proto/bank.proto"], &["proto"])
        .expect("Failed to compile protobufs");
}
