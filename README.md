# Rust-SAF
A Rust library providing an API accessing files with Android Storage Access Framework (SAF).

## Usage

Currently this library is not published on crates.io. Current available trait and struct should be stable, but the impl block is not stable. The library it self is located in `app/src/main/rust/ndk-saf` directory, and the whole project is an Android project to test this library.

## Steps to test this lib yourself

1. Cloning this repository and opening it with Android Studio.
2. Setup NDK and CMake in Android Studio.
3. Specify `ndk.dir` in `local.properties` file.
4. Build and voila!

This project uses [cargo-ndk-android-gradle](https://github.com/willir/cargo-ndk-android-gradle) to integrate Rust code with Android project. The `Cargo.toml` file is located in `app/src/main/rust` directory. You may check that out if anything is not working.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
