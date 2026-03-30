plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "com.example.sampleappwithrust"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        applicationId = "com.example.sampleappwithrust"
        minSdk = 35
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
    }
}

// -----------------------------------------------------------------------
// Rust JNI ライブラリのビルド（cargo ndk を直接呼び出す）
//
// rust-android-gradle プラグイン (0.9.6) は AGP 9.x と非互換のため、
// cargo ndk を Exec タスクで直接実行する方式を採用。
//
// 事前準備:
//   cargo install cargo-ndk
//   rustup target add aarch64-linux-android x86_64-linux-android
// -----------------------------------------------------------------------

val cargoNdkBuildDebug by tasks.registering(Exec::class) {
    group = "rust"
    description = "Build Rust JNI library (debug) via cargo ndk"
    workingDir = file("../rust")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", "../app/src/main/jniLibs",
        "build"
    )
}

val cargoNdkBuildRelease by tasks.registering(Exec::class) {
    group = "rust"
    description = "Build Rust JNI library (release) via cargo ndk"
    workingDir = file("../rust")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", "../app/src/main/jniLibs",
        "build", "--release"
    )
}

// デバッグビルド前に Rust ライブラリをビルドする
tasks.named("preBuild") {
    dependsOn(cargoNdkBuildDebug)
}

// リリースビルド前はリリース版 Rust ライブラリを使う
tasks.whenTaskAdded {
    if (name == "mergeReleaseJniLibFolders") {
        dependsOn(cargoNdkBuildRelease)
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.tooling)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}
