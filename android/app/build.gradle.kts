plugins {
    id("com.android.application")
}

android {
    namespace = "com.ifpf.vitriol"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.ifpf.vitriol"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "x86_64"))
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            assets.srcDirs("../../assets")
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    bundle {
        language {
            enableSplit = false
        }
        density {
            enableSplit = true
        }
        abi {
            enableSplit = true
        }
    }
}

dependencies {
    implementation("androidx.core:core:1.13.1")
    implementation("androidx.games:games-activity:3.0.4")
}
