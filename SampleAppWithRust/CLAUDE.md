# Android × Rust × Protobuf 統合パターン

このプロジェクトは、Android アプリから Rust の JNI ライブラリを呼び出し、
Protobuf でシリアライズしたデータを非同期コールバックでやり取りするパターンの検証用。
同様の構成を別プロジェクトに適用する際の手順と注意点をまとめる。

---

## フォルダ構成

```
<project-root>/
├── app/                          # Android アプリモジュール
├── proto/                        # .proto ファイル（Rust・Android 共通ソース）
│   └── audio_init.proto
├── rust/                         # Rust ワークスペース
│   ├── Cargo.toml                # [workspace] members = ["audio_core", "jni_bridge"]
│   ├── audio_core/               # ライブラリ①: ビジネスロジック（JNI 非依存）
│   └── jni_bridge/               # ライブラリ②: JNI エントリーポイント
└── CLAUDE.md
```

---

## Gradle 設定

### 注意: AGP 9.x とプラグインの互換性

AGP 9.x では以下の 2 つのプラグインが**どちらも非互換**:

| プラグイン | エラー |
|---|---|
| `com.google.protobuf` 0.9.x | `Cannot cast object 'com.android.build.g...'` |
| `org.mozilla.rust-android-gradle` 0.9.6 | `Extension of type 'AppExtension' does not exist` |

両プラグインとも AGP の内部 API（`AppExtension`）が 9.x で廃止されたことが原因。

**対処**:
- proto: `ProtoHelper.kt` で手動エンコード/デコード
- Rust ビルド: `cargo ndk` を `Exec` タスクで直接呼び出す

### build.gradle.kts（ルート）

追加なし（buildscript 不要）。標準の plugins ブロックのみ:

```kotlin
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.compose) apply false
}
```

### app/build.gradle.kts

```kotlin
plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    // rust-android-gradle / protobuf-gradle-plugin は AGP 9.x と非互換のため追加しない
}

android {
    // ...既存設定...
}

// Rust ビルド: cargo ndk を Exec タスクで直接呼び出す
val cargoNdkBuildDebug by tasks.registering(Exec::class) {
    workingDir = file("../rust")
    commandLine("cargo", "ndk", "-t", "arm64-v8a", "-t", "x86_64",
                "-o", "../app/src/main/jniLibs", "build")
}

tasks.named("preBuild") {
    dependsOn(cargoNdkBuildDebug)
}

dependencies {
    // ...既存依存...
    // protobuf-kotlin-lite は不要（ProtoHelper で手動実装）
}
```

---

## Rust ワークスペース構成

### rust/Cargo.toml

```toml
[workspace]
members = ["audio_core", "jni_bridge"]
resolver = "2"
```

### ライブラリ①: audio_core（ビジネスロジック層）

- `crate-type = ["rlib"]`（Android に直接 .so として出力しない）
- JNI 依存なし。コールバック型は `Box<dyn Fn(Vec<u8>) + Send + 'static>`
- proto のデコード（リクエスト）・エンコード（レスポンス）はこの層で行う

**Cargo.toml の依存:**
```toml
[dependencies]
prost = "0.12"

[build-dependencies]
prost-build = "0.12"
protoc-bin-vendored = "3"   # システムへの protoc インストール不要にするため
```

**build.rs:**
```rust
fn main() {
    let protoc_path = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc_path);
    prost_build::compile_protos(
        &["../../proto/audio_init.proto"],
        &["../../proto/"],
    ).unwrap();
}
```

**生成コードのインクルード（lib.rs）:**
```rust
pub mod audio_proto {
    include!(concat!(env!("OUT_DIR"), "/audio.rs"));  // package名がファイル名になる
}
```

### ライブラリ②: jni_bridge（JNI 層）

- `crate-type = ["cdylib"]`、`name = "mylib"`（Gradle の `libname` と一致）
- proto の変換は不要。バイト列をそのまま受け渡しする
- `audio_core` をパス依存として参照

**Cargo.toml の依存:**
```toml
[lib]
name = "mylib"          # ← Gradle libname と一致させること
crate-type = ["cdylib"]

[dependencies]
audio_core = { path = "../audio_core" }
jni = "0.21"
```

---

## JNI 関数の命名規則

```
Java_{パッケージ名（.を_に）}_{クラス名}_{メソッド名}
```

例: パッケージ `com.example.myapp`、クラス `RustBridge`、メソッド `nativeInit`
→ `Java_com_example_myapp_RustBridge_nativeInit`

Kotlin 側の定義:
```kotlin
object RustBridge {
    init { System.loadLibrary("mylib") }
    external fun nativeInit(data: ByteArray, callback: AudioCallback)
}
```

Rust 側の定義（第2引数は `JObject`、staticでなければ instanceオブジェクト）:
```rust
#[no_mangle]
pub extern "system" fn Java_com_example_myapp_RustBridge_nativeInit(
    mut env: JNIEnv,
    _obj: JObject,
    data: JByteArray,
    callback: JObject,
) { ... }
```

---

## スレッド間コールバックパターン

JNI からバックグラウンドスレッドで Java を呼ぶための定型パターン:

```rust
// 1. JavaVM と GlobalRef をスレッド境界を越えて保持
let jvm = env.get_java_vm().unwrap();
let callback_global = env.new_global_ref(&callback).unwrap();

// 2. チャンネルで Rust スレッド → Java 呼び出しスレッドへ渡す
let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

// 3. 受信スレッド: JVM にアタッチして Java メソッドを呼ぶ
std::thread::spawn(move || {
    let bytes = rx.recv().unwrap();
    let mut env = jvm.attach_current_thread().unwrap();
    let java_bytes = env.byte_array_from_slice(&bytes).unwrap();
    env.call_method(&callback_global, "onCallback", "([B)V",
        &[JValue::Object(&java_bytes)]).unwrap();
});

// 4. Rust コールバック: チャンネル送信のみ（JVM 操作なし）
some_lib_init(Box::new(move |result| { tx.send(result).unwrap(); }), ...);
```

Kotlin 側での受信（Compose + メインスレッド dispatch）:
```kotlin
val mainHandler = remember { Handler(Looper.getMainLooper()) }

RustBridge.nativeInit(requestBytes) { responseBytes ->
    // バックグラウンドスレッドから呼ばれる
    val response = AudioCallbackResponse.parseFrom(responseBytes)
    mainHandler.post {
        // Compose の状態はメインスレッドから更新する
        versionText = response.version
    }
}
```

---

## Kotlin 側の proto 手動エンコード/デコード（ProtoHelper）

AGP 9.x では protobuf Gradle プラグインが使えないため、`ProtoHelper.kt` で代替する。
wire format のルール（varint + length-delimited）を直接実装する。

```kotlin
// エンコード例（string フィールド）
fun encodeAudioInitRequest(versionString: String): ByteArray {
    // field 1, wire type 2 → tag = 0x0A
    val strBytes = versionString.toByteArray(Charsets.UTF_8)
    return byteArrayOf(0x0A) + encodeVarint(strBytes.size.toLong()) + strBytes
}

// デコード例（varint で長さを読んで文字列を取り出す）
fun decodeAudioCallbackResponse(bytes: ByteArray): AudioCallbackResponse {
    // tag から field番号・wire type を取り出し、文字列フィールドを読む
}
```

完全な実装は [ProtoHelper.kt](app/src/main/java/com/example/sampleappwithrust/ProtoHelper.kt) を参照。
将来 AGP 9.x 対応版の protobuf プラグインがリリースされたら置き換え可能。

---

## Proto ファイルの設計指針

- `java_package` / `java_multiple_files = true` を必ず指定（Android 側の import パスを明確にする）
- `oneof` は排他的な設定値に使用する（初期化パラメータのバリエーションなど）
- リクエストとレスポンスで別メッセージを定義する

```proto
syntax = "proto3";
package audio;

option java_package = "com.example.myapp.proto";
option java_multiple_files = true;

message MyRequest {
  oneof spec {
    string string_value = 1;
    uint32 number_value = 2;
  }
}

message MyResponse {
  string version = 1;
  string message = 2;
}
```

---

## 事前準備（ビルド環境）

```bash
# Android ターゲットの追加
rustup target add aarch64-linux-android x86_64-linux-android

# cargo-ndk のインストール（Mozilla プラグインが内部使用）
cargo install cargo-ndk
```

- **NDK**: Android Studio → SDK Manager → SDK Tools → NDK (Side by side) でインストール
- **protoc**: `protoc-bin-vendored` クレートを使えばインストール不要
- **local.properties** に `ndk.dir` が設定されていることを確認（Studio が自動生成）
