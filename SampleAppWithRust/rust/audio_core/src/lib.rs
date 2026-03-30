use prost::Message;
use std::thread;
use std::time::Duration;

// prost_build が生成したコードをインクルード
pub mod audio_proto {
    include!(concat!(env!("OUT_DIR"), "/audio.rs"));
}

/// コールバック型: 初期化完了後に AudioCallbackResponse の proto エンコード済みバイト列を渡す
pub type AudioCallback = Box<dyn Fn(Vec<u8>) + Send + 'static>;

/// 音認コア初期化（モック実装）
///
/// # 引数
/// - `callback`: 初期化完了時に呼ばれるコールバック（proto エンコード済みバイト列を受け取る）
/// - `request_bytes`: AudioInitRequest の proto エンコード済みバイト列
///
/// # 動作
/// 別スレッドを起動し、10秒後に callback を呼ぶ（非同期）
pub fn audio_core_init(callback: AudioCallback, request_bytes: Vec<u8>) {
    // リクエストをデコードしてバージョン文字列を取得
    let request = audio_proto::AudioInitRequest::decode(request_bytes.as_slice())
        .unwrap_or_default();

    let initial_version = match request.version_spec {
        Some(audio_proto::audio_init_request::VersionSpec::VersionString(s)) => s,
        Some(audio_proto::audio_init_request::VersionSpec::VersionNumber(n)) => {
            format!("{}.0.0", n)
        }
        None => "1.0.0".to_string(),
    };

    // 別スレッドで 10 秒後にコールバック
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(10));

        let response = audio_proto::AudioCallbackResponse {
            version: initial_version,
            message: "Audio core initialization complete".to_string(),
        };

        let mut buf = Vec::new();
        response.encode(&mut buf).expect("Failed to encode AudioCallbackResponse");
        callback(buf);
    });
}
