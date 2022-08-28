package io.github.markusjx.bridge;

@SuppressWarnings("unused")
public class NativeLibrary {
    public static void loadLibrary(String nativePath) {
        Runtime.getRuntime().load(nativePath.replace('\\', '/'));
    }
}