fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/spark_connect.proto"], &["proto"])
        .expect("compile spark connect proto");
}
