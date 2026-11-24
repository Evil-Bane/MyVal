import java.io.File
import java.util.Properties

plugins {
  alias(libs.plugins.android.application)
  alias(libs.plugins.compose.compiler)
  alias(libs.plugins.kotlin.serialization)
}

val keystorePropsFile = rootProject.file("keystore.properties")
val keystoreProps = Properties().apply {
  if (keystorePropsFile.exists()) {
    keystorePropsFile.inputStream().use { load(it) }
  }
}

fun releaseSigningProp(propKey: String, envKey: String): String? =
  keystoreProps.getProperty(propKey)?.takeIf { it.isNotBlank() }
    ?: System.getenv(envKey)?.takeIf { it.isNotBlank() }

android {
  namespace = "com.myval.app"
  compileSdk {
    version = release(36) {
      minorApiLevel = 1
    }
  }
  defaultConfig {
    applicationId = "com.myval.app"
    minSdk = 26
    targetSdk = 36
    versionCode = 17
    versionName = "1.0.6"
    buildConfigField("String", "GITHUB_REPO", "\"Evil-Bane/MyVal\"")
    ndk {
      abiFilters += listOf("arm64-v8a")
    }
  }

  signingConfigs {
    val storePath = releaseSigningProp("storeFile", "KEYSTORE_PATH")
    val storePass = releaseSigningProp("storePassword", "KEYSTORE_PASSWORD")
    val alias = releaseSigningProp("keyAlias", "KEY_ALIAS")
    val keyPass = releaseSigningProp("keyPassword", "KEY_PASSWORD")
    val resolvedStoreFile = storePath?.let { rootProject.file(it) }?.takeIf { it.exists() }
    if (resolvedStoreFile != null && storePass != null && alias != null && keyPass != null) {
      create("release") {
        storeFile = resolvedStoreFile
        storePassword = storePass
        keyAlias = alias
        keyPassword = keyPass
        enableV1Signing = true
        enableV2Signing = true
        enableV3Signing = true
      }
    }
  }

  buildTypes {
    release {
      isMinifyEnabled = true
      isShrinkResources = true
      proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
      signingConfig = signingConfigs.findByName("release") ?: signingConfigs.getByName("debug")
    }
  }
  compileOptions {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
  }
  buildFeatures {
    compose = true
    aidl = false
    buildConfig = true
    shaders = false
  }

  packaging {
    resources {
      excludes += "/META-INF/{AL2.0,LGPL2.1}"
    }
    jniLibs {
      useLegacyPackaging = false
    }
  }
}

abstract class VerifyUniFfiTask : org.gradle.api.DefaultTask() {
  @get:org.gradle.api.tasks.Input
  abstract val expectedFilePath: org.gradle.api.provider.Property<String>

  @org.gradle.api.tasks.TaskAction
  fun verify() {
    val ffiFile = File(expectedFilePath.get())
    if (!ffiFile.exists()) {
      throw org.gradle.api.GradleException(
        "Missing generated Rust UniFFI bindings at ${ffiFile.path}. " +
          "Run scripts/build-android-rust.ps1 (Windows) or scripts/build-android-rust.sh (macOS/Linux) first."
      )
    }
  }
}

val verifyRustBindings = tasks.register<VerifyUniFfiTask>("verifyRustBindings") {
  expectedFilePath.set(
    layout.projectDirectory.file("src/main/java/uniffi/myval_ffi/myval_ffi.kt").asFile.absolutePath
  )
}

tasks.matching { it.name == "preBuild" }.configureEach {
  dependsOn(verifyRustBindings)
}

kotlin {
  compilerOptions {
    jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
  }
}

dependencies {
  val composeBom = platform(libs.androidx.compose.bom)
  implementation(composeBom)
  androidTestImplementation(composeBom)

  // Core Android dependencies
  implementation(libs.androidx.core.ktx)
  implementation(libs.androidx.lifecycle.runtime.ktx)
  implementation(libs.androidx.activity.compose)
  implementation(libs.kotlinx.coroutines.android)

  // Arch Components
  implementation(libs.androidx.lifecycle.runtime.compose)
  implementation(libs.androidx.lifecycle.viewmodel.compose)

  // Compose
  implementation(libs.androidx.compose.ui)
  implementation(libs.androidx.compose.ui.tooling.preview)
  implementation(libs.androidx.compose.material3)
  implementation(libs.androidx.compose.material.icons.extended)

  // UniFFI JNA native bridge + Image Loading
  implementation("net.java.dev.jna:jna:5.15.0@aar")
  implementation(libs.coil.compose)
  implementation("com.squareup.okhttp3:okhttp:4.12.0")

  // Tooling
  debugImplementation(libs.androidx.compose.ui.tooling)
  // Instrumented tests
  androidTestImplementation(libs.androidx.compose.ui.test.junit4)
  debugImplementation(libs.androidx.compose.ui.test.manifest)

  // Local tests: jUnit, coroutines, org.json JVM runtime
  testImplementation(libs.junit)
  testImplementation(libs.kotlinx.coroutines.test)
  testImplementation("org.json:json:20240303")

  // Instrumented tests: jUnit rules and runners
  androidTestImplementation(libs.androidx.test.core)
  androidTestImplementation(libs.androidx.test.ext.junit)
  androidTestImplementation(libs.androidx.test.runner)
  androidTestImplementation(libs.androidx.test.espresso.core)

  // Navigation
  implementation(libs.androidx.navigation3.ui)
  implementation(libs.androidx.navigation3.runtime)
  implementation(libs.androidx.lifecycle.viewmodel.navigation3)
}
