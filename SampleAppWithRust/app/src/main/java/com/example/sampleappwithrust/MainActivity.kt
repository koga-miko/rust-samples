package com.example.sampleappwithrust

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.sampleappwithrust.ui.theme.SampleAppWithRustTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            SampleAppWithRustTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    AudioInitScreen(modifier = Modifier.padding(innerPadding))
                }
            }
        }
    }
}

@Composable
fun AudioInitScreen(modifier: Modifier = Modifier) {
    var versionText by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }
    val mainHandler = remember { Handler(Looper.getMainLooper()) }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        if (versionText.isNotEmpty()) {
            Text(
                text = "バージョン: $versionText",
                style = MaterialTheme.typography.headlineMedium,
            )
            Spacer(modifier = Modifier.height(24.dp))
        }

        if (isLoading) {
            CircularProgressIndicator()
            Spacer(modifier = Modifier.height(16.dp))
            Text(text = "初期化中... (約10秒お待ちください)")
            Spacer(modifier = Modifier.height(16.dp))
        }

        Button(
            onClick = {
                isLoading = true
                versionText = ""

                // AudioInitRequest を proto エンコードして Rust (JNI) に渡す
                val requestBytes = ProtoHelper.encodeAudioInitRequest("2.0.0")

                RustBridge.nativeInit(requestBytes) { responseBytes ->
                    // Rust スレッドから呼ばれる（バックグラウンドスレッド）
                    Log.d("AudioInit", "callback called on thread: ${Thread.currentThread().name}")
                    // AudioCallbackResponse をデコードしてバージョン文字列を取得
                    val response = ProtoHelper.decodeAudioCallbackResponse(responseBytes)
                    Log.d("AudioInit", "callback response: version=${response.version}, message=${response.message}")
                    mainHandler.post {
                        versionText = response.version
                        isLoading = false
                    }
                }
                Log.d("AudioInit", "nativeInit returned (non-blocking) on thread: ${Thread.currentThread().name}")
            },
            enabled = !isLoading,
        ) {
            Text(text = if (isLoading) "初期化中..." else "音認コアを初期化")
        }
    }
}
