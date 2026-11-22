plugins {
  id("com.android.application")
  id("org.jetbrains.kotlin.android")
  id("org.jetbrains.kotlin.plugin.compose")
}

android {
  namespace = "com.wfhub.cagatayldzz"
  compileSdk = 36

  defaultConfig {
    applicationId = "com.wfhub.cagatayldzz"
    minSdk = 26
    targetSdk = 36
    versionCode = 1
    versionName = "2.2"
    // The Rust core (libwfhub_core.so) is built by `bash scripts/build-core.sh android` into src/main/jniLibs.
    ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64") }
  }

  buildTypes {
    release {
      isMinifyEnabled = true
      isShrinkResources = true
      proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
    }
  }

  compileOptions {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
  }
  buildFeatures { compose = true }

  sourceSets {
    // UniFFI's Kotlin bindings of the Rust core (made by scripts/build-core.sh).
    getByName("main").java.srcDir("src/main/generated")
  }
  packaging {
    jniLibs.useLegacyPackaging = false
    resources.excludes += setOf("META-INF/AL2.0", "META-INF/LGPL2.1")
  }
}

kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }

dependencies {
  implementation(platform("androidx.compose:compose-bom:2025.10.01"))
  implementation("androidx.core:core-ktx:1.17.0")
  implementation("androidx.activity:activity-compose:1.11.0")
  implementation("androidx.lifecycle:lifecycle-runtime-compose:2.9.4")
  implementation("androidx.navigation:navigation-compose:2.9.5")
  implementation("androidx.compose.ui:ui")
  implementation("androidx.compose.foundation:foundation")
  implementation("androidx.compose.material3:material3")
  implementation("androidx.compose.material:material-icons-extended")
  implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
  implementation("io.coil-kt.coil3:coil-compose:3.3.0")
  implementation("io.coil-kt.coil3:coil-network-okhttp:3.3.0")
  // UniFFI's Kotlin bindings talk to the Rust library through JNA.
  implementation("net.java.dev.jna:jna:5.19.1@aar")

  debugImplementation("androidx.compose.ui:ui-tooling")
  testImplementation("junit:junit:4.13.2")
}
