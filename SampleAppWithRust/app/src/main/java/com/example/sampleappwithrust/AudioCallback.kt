package com.example.sampleappwithrust

/**
 * Rust の JNI ブリッジから呼ばれるコールバックインターフェース。
 * [data] は AudioCallbackResponse の proto エンコード済みバイト列。
 */
fun interface AudioCallback {
    fun onCallback(data: ByteArray)
}
