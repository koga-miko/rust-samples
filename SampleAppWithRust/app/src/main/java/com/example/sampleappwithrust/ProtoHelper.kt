package com.example.sampleappwithrust

/**
 * protobuf-gradle-plugin が AGP 9.x と非互換のため、
 * AudioInitRequest / AudioCallbackResponse の最小限の手動エンコード/デコードを実装する。
 *
 * proto 定義（proto/audio_init.proto）との対応:
 *
 *   message AudioInitRequest {
 *     oneof version_spec {
 *       string version_string = 1;   // tag = 0x0A
 *       uint32 version_number = 2;   // tag = 0x10
 *     }
 *   }
 *
 *   message AudioCallbackResponse {
 *     string version = 1;            // tag = 0x0A
 *     string message = 2;            // tag = 0x12
 *   }
 */
object ProtoHelper {

    // ---- エンコード --------------------------------------------------------

    /**
     * AudioInitRequest(version_string = [value]) をエンコードする。
     * wire format: tag(0x0A) + varint(length) + UTF-8 bytes
     */
    fun encodeAudioInitRequest(versionString: String): ByteArray {
        val strBytes = versionString.toByteArray(Charsets.UTF_8)
        return buildProtoBytes {
            writeStringField(fieldNumber = 1, value = strBytes)
        }
    }

    /**
     * AudioInitRequest(version_number = [value]) をエンコードする。
     * wire format: tag(0x10) + varint(value)
     */
    fun encodeAudioInitRequest(versionNumber: UInt): ByteArray {
        return buildProtoBytes {
            writeVarintField(fieldNumber = 2, value = versionNumber.toLong())
        }
    }

    // ---- デコード ----------------------------------------------------------

    data class AudioCallbackResponse(val version: String, val message: String)

    /**
     * AudioCallbackResponse をデコードして version と message を返す。
     */
    fun decodeAudioCallbackResponse(bytes: ByteArray): AudioCallbackResponse {
        var version = ""
        var message = ""
        var i = 0
        while (i < bytes.size) {
            val (fieldNumber, wireType, nextIndex) = readTag(bytes, i)
            i = nextIndex
            when (wireType) {
                2 -> { // length-delimited (string)
                    val (str, afterIndex) = readString(bytes, i)
                    i = afterIndex
                    when (fieldNumber) {
                        1 -> version = str
                        2 -> message = str
                    }
                }
                else -> break // 未知のフィールドは無視（本来はスキップ処理が必要）
            }
        }
        return AudioCallbackResponse(version, message)
    }

    // ---- 内部ユーティリティ ------------------------------------------------

    private class ProtoBuilder {
        private val buf = mutableListOf<Byte>()

        fun writeStringField(fieldNumber: Int, value: ByteArray) {
            writeTag(fieldNumber, wireType = 2)
            writeVarint(value.size.toLong())
            value.forEach { buf.add(it) }
        }

        fun writeVarintField(fieldNumber: Int, value: Long) {
            writeTag(fieldNumber, wireType = 0)
            writeVarint(value)
        }

        private fun writeTag(fieldNumber: Int, wireType: Int) {
            writeVarint(((fieldNumber shl 3) or wireType).toLong())
        }

        private fun writeVarint(value: Long) {
            var v = value
            while (true) {
                val b = (v and 0x7F).toInt()
                v = v ushr 7
                if (v == 0L) {
                    buf.add(b.toByte())
                    break
                } else {
                    buf.add((b or 0x80).toByte())
                }
            }
        }

        fun toByteArray() = buf.toByteArray()
    }

    private fun buildProtoBytes(block: ProtoBuilder.() -> Unit): ByteArray =
        ProtoBuilder().apply(block).toByteArray()

    private data class TagResult(val fieldNumber: Int, val wireType: Int, val nextIndex: Int)

    private fun readTag(bytes: ByteArray, start: Int): TagResult {
        val (tagValue, nextIndex) = readVarint(bytes, start)
        return TagResult(
            fieldNumber = (tagValue ushr 3).toInt(),
            wireType = (tagValue and 0x07).toInt(),
            nextIndex = nextIndex,
        )
    }

    private fun readString(bytes: ByteArray, start: Int): Pair<String, Int> {
        val (length, afterLength) = readVarint(bytes, start)
        val end = afterLength + length.toInt()
        return Pair(String(bytes, afterLength, length.toInt(), Charsets.UTF_8), end)
    }

    /** varint を読んで (値, 次のインデックス) を返す */
    private fun readVarint(bytes: ByteArray, start: Int): Pair<Long, Int> {
        var result = 0L
        var shift = 0
        var i = start
        while (i < bytes.size) {
            val b = bytes[i++].toInt() and 0xFF
            result = result or ((b and 0x7F).toLong() shl shift)
            if (b and 0x80 == 0) break
            shift += 7
        }
        return Pair(result, i)
    }
}
