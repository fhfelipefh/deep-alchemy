-keep class android.app.NativeActivity { *; }
-keep class com.google.androidgamesdk.** { *; }
-keep class androidx.games.** { *; }
-keepclassmembers class * {
    native <methods>;
}
-dontwarn android.app.NativeActivity
