# Directory listing regression

`ListingTest` calls the Rust crate against an Android content provider. The
test JNI library is an example target and is not part of the sample app.
From the repository root, with an x86_64 emulator running, the Android NDK
and cargo-ndk installed:

```sh
cd app/src/main/rust
cargo ndk -t x86_64 build --locked -p ndk-saf --example listing_test
mkdir -p ../../androidTest/jniLibs/x86_64
cp target/x86_64-linux-android/debug/examples/liblisting_test.so ../../androidTest/jniLibs/x86_64/
cd ../../../..
./gradlew connectedDebugAndroidTest
```
