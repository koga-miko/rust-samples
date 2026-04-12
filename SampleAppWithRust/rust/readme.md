# Visual Studio Codeから　Android アプリのRustコードをデバッグする方法

## 準備

### Android NDKのインストール
1. adb コマンドのインストール
Android Studio の　Files > Settings > 検索ボックスで"SDK" > Android SDK > SDK Tools で、「Android SDK Platform-Tools」にチェックが入っていなければ、チェックしてインストール。
※チェックが入っていなければ、この手順は不要。

2. adb コマンドのパスを通す
コマンドプロンプトで、"where adb"でパスが出れば、adbコマンドが使える状態だが、
でなければ、環境変数のPathに、以下を追加する必要がある。
```
%LOCALAPPDATA%\Android\Sdk\platform-tools
```

3. NDKのインストール
手順1記載のSDK Tools タブを開き、NDKが入っていなければNDKをチェックしていれる
※バージョンは、ひとまず新しめのものであればOK。

4. NDKの中から、lldb-serverというファイルを探す
候補：
```
"%LOCALAPPDATA%\Android\Sdk\ndk\<version>\toolchains\llvm\prebuilt\windows-x86_64\lib\clang\<clang-version>\lib\linux\x86_64\"
```

5. lldb-serverをエミュレータに転送して起動
:: エミュレータに転送
```
adb push "%LOCALAPPDATA%\Android\Sdk\ndk\<version>\toolchains\llvm\prebuilt\windows-x86_64\lib\clang\<clang-version>\lib\linux\x86_64\lldb-server" /data/local/tmp/
```

:: 実行権限を付与
```
adb shell chmod +x /data/local/tmp/lldb-server
```

:: lldb-serverを起動
```
adb shell /data/local/tmp/lldb-server platform --listen "*:12345" --server
```

6. 別のターミナルを開いて、ローカルの12345ポートが、エミュレータのlldb-serverへ転送されるように設定。
```
adb forward tcp:12345 tcp:12345
```

7. VS Code のlaunch.jsonの設定
VS Codeで、 Ctrl+Shift+P > "Debug: Open launch.json"
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "attach",
            "name": "Android Rust Debug",
            "pid": "${command:pickProcess}",
            "initCommands": [
                "platform select remote-android",
                "platform connect connect://localhost:12345"
            ],
            "sourceMap": {
                "/buildbot/src": "${workspaceFolder}"
            }
        }
    ]
}
```

8. アタッチするプロセスのPIDを確認
```
adb shell ps -A | findstr "com.example"
```

例）以下の場合は、3447が、PIDになる。
```
u0_a150       3447   385   15696196 139220 futex_wait_queue    0 S com.example.sampleappwithrust
```

9. VS CodeからデバッガをAttach
  - VS Codeで Run and Debug パネルを開く（Ctrl+Shift+D）
  - 上部のドロップダウンで Android Rust Debug を選択
  - ▶ ボタンを押す
  - プロセス選択のダイアログが出たら 3447 を入力またはリストから選択

例） Attach成功のサイン
- VS Code下部のステータスバーが変わる
オレンジ色になる
- Run and Debugパネルに情報が表示される
CALL STACK ペインにスレッド一覧が表示される
例：
  Thread 1 (main)
  Thread 2 ...
- デバッグツールバーが出る
画面上部に ⏸ ▶ ⏭ などのボタンが出る

10. Rustファイルにブレークポイントを張る
該当の.rsファイルをVS Codeで開いて、行番号の左をクリックして赤丸をつけて止まるか試す。


### 補足：今回のまとめ
❌ Android Studio + Google Play版エミュレータ
      → LLDBがAttachできない

❌ Android Studio + AOSP版エミュレータ
      → RustファイルへのBPが非対応

✅ VS Code + CodeLLDB + AOSP版エミュレータ
      → Rustソースレベルデバッグ成功！

### 補足：今後のデバッグ手順（定常作業）
```
:: 1. lldb-server起動
adb shell /data/local/tmp/lldb-server platform --listen "*:12345" --server

:: 2. 別ターミナルでポートフォワード
adb forward tcp:12345 tcp:12345

:: 3. アプリ起動後にVS CodeからAttach
```
シェルスクリプトにまとめておくと楽。