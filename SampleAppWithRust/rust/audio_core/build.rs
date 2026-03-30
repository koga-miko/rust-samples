fn main() {
    // protoc バイナリをバンドルして使用（システムへの protoc インストール不要）
    let protoc_path = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc_path);

    prost_build::compile_protos(
        &["../../proto/audio_init.proto"],
        &["../../proto/"],
    )
    .unwrap();
}
