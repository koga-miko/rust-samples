package com.example.sampleappwithrust

/**
 * Rust の JNI ブリッジへの Kotlin インターフェース。
 * libmylib.so をロードし、nativeInit を公開する。
 */
object RustBridge {

    init {
        System.loadLibrary("mylib")
    }

    /**
     * 音認コア初期化の JNI 呼び出し。
     *
     * @param data     AudioInitRequest の proto エンコード済みバイト列
     * @param callback 10 秒後に AudioCallbackResponse の proto バイト列を受け取るコールバック
     */
    external fun nativeInit(data: ByteArray, callback: AudioCallback)
}
