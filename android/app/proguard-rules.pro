# ==============================================================================
# MyVal Release R8 / ProGuard Configuration
# Obfuscates com.myval.app.** while keeping JNA / UniFFI native FFI intact.
# ==============================================================================

-keepattributes *Annotation*,Signature,InnerClasses,EnclosingMethod

# JNA + UniFFI native bridge (Rust <-> Kotlin FFI)
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class * extends com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.Structure {
    public <fields>;
    public <init>(...);
}
-keep class uniffi.myval_ffi.** { *; }
-dontwarn com.sun.jna.**
-dontwarn java.awt.**

# AndroidX ViewModel constructor reflection
-keepclassmembers class * extends androidx.lifecycle.AndroidViewModel {
    public <init>(android.app.Application);
}
-keepclassmembers class * extends androidx.lifecycle.ViewModel {
    public <init>(...);
}

# Strip verbose debug logging in release builds
-assumenosideeffects class android.util.Log {
    public static boolean isLoggable(java.lang.String, int);
    public static int v(...);
    public static int d(...);
    public static int i(...);
}
