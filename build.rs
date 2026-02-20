fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/spark/connect/base.proto"], &["proto"])
        .expect("failed to compile spark connect protos");
}
