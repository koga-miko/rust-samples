use jni::objects::{JByteArray, JClass, JObject, JValue};
use jni::JNIEnv;
use std::sync::mpsc;

/// JNI エントリーポイント
///
/// Java 側の RustBridge.nativeInit(byte[], AudioCallback) に対応。
///
/// # 処理フロー
/// 1. コールバック受信スレッドを起動（チャンネルから受信待機）
/// 2. Java から受け取ったバイト列（AudioInitRequest の proto）を audio_core_init に渡す
/// 3. 10 秒後に audio_core からコールバックが来たらチャンネル送信
/// 4. 受信スレッドが proto エンコード済みバイト列を Java の onCallback に渡す
#[no_mangle]
pub extern "system" fn Java_com_example_sampleappwithrust_RustBridge_nativeInit(
    mut env: JNIEnv,
    _obj: JObject,
    data: JByteArray,
    callback: JObject,
) {
    // JavaVM への参照を取得（スレッド間で共有するため）
    let jvm = match env.get_java_vm() {
        Ok(vm) => vm,
        Err(e) => {
            eprintln!("[jni_bridge] Failed to get JavaVM: {:?}", e);
            return;
        }
    };

    // callback オブジェクトのグローバル参照を作成（スレッドを越えて保持するため）
    let callback_global = match env.new_global_ref(&callback) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("[jni_bridge] Failed to create global ref: {:?}", e);
            return;
        }
    };

    // Rust 内部チャンネル: audio_core のコールバックスレッド → Java 呼び出しスレッド
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // コールバック受信スレッド:
    // audio_core からの proto バイト列を受け取り、Java の onCallback を呼ぶ
    std::thread::spawn(move || {
        match rx.recv() {
            Ok(response_bytes) => {
                let mut env = match jvm.attach_current_thread() {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("[jni_bridge] Failed to attach thread to JVM: {:?}", e);
                        return;
                    }
                };

                // Vec<u8> を Java の byte[] に変換
                let java_bytes = match env.byte_array_from_slice(&response_bytes) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("[jni_bridge] Failed to create byte array: {:?}", e);
                        return;
                    }
                };

                // Java の AudioCallback#onCallback(byte[]) を呼ぶ
                if let Err(e) = env.call_method(
                    &callback_global,
                    "onCallback",
                    "([B)V",
                    &[JValue::Object(&java_bytes)],
                ) {
                    eprintln!("[jni_bridge] Failed to call Java callback: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("[jni_bridge] Channel receive error: {:?}", e);
            }
        }
    });

    // Java から渡されたバイト列（AudioInitRequest の proto）を取得
    let request_bytes = match env.convert_byte_array(&data) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[jni_bridge] Failed to convert byte array: {:?}", e);
            return;
        }
    };

    // audio_core 初期化呼び出し
    // コールバック内でチャンネル送信する（受信スレッドが Java を呼ぶ）
    audio_core::audio_core_init(
        Box::new(move |response_bytes| {
            if let Err(e) = tx.send(response_bytes) {
                eprintln!("[jni_bridge] Channel send error: {:?}", e);
            }
        }),
        request_bytes,
    );
}
