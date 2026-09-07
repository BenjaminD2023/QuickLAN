-keep class io.quicklan.android.NativeBridge { *; }
-keep class io.quicklan.android.AndroidPlatform { *; }
-keepclassmembers class io.quicklan.android.NativeBridge {
    public *;
}
-keepclassmembers class io.quicklan.android.AndroidPlatform {
    public *;
}
