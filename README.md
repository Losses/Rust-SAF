# Rust-SAF
A Rust library providing an API accessing files with Android Storage Access Framework (SAF).

## Usage

Currently this library is not published on crates.io. Current available trait and struct should be stable, but the impl block is not stable. The library it self is located in `app/src/main/rust/ndk-saf` directory, and the whole project is an Android project to test this library.

### Core Concepts

The two main components of this library are the `AndroidFile` struct and the `AndroidFileOps` trait.

#### `AndroidFile` Struct

The `AndroidFile` struct represents a file or directory accessible through the Storage Access Framework. It holds metadata about the file and an internal reference to the Java `DocumentFile` object.

```rust
pub struct AndroidFile {
    pub filename: String,     // File name
    pub size: usize,          // File size in bytes, behavior undefined for directories
    pub path: String,         // Path (not valid path, only for display)
    pub url: String,          // Content URI (use THIS to obtain the AndroidFile object again)
    pub is_dir: bool,         // Is the file a directory
    document_file: GlobalRef, // JNI DocumentFile JObject representing the file
}
```

- `filename`: The name of the file or directory.
- `size`: The size of the file in bytes. The behavior is undefined for directories.
- `path`: A display path, not a true file system path.
- `url`: The content URI for the file or directory. This is the primary identifier and can be used to re-create an `AndroidFile` object.
- `is_dir`: A boolean indicating if the `AndroidFile` represents a directory.
- `document_file`: An internal JNI global reference to the underlying `androidx.documentfile.provider.DocumentFile` Java object.

#### `AndroidFileOps` Trait

The `AndroidFileOps` trait defines the set of operations that can be performed on an `AndroidFile` object.

```rust
pub trait AndroidFileOps {
    fn open(&self, open_mode: &str) -> Result<File>;
    fn list_files(&self) -> Result<Vec<AndroidFile>>;
    fn create_file(&self, mime_type: &str, file_name: &str) -> Result<AndroidFile>;
    fn create_directory(&self, dir_name: &str) -> Result<AndroidFile>;
    fn remove_file(&self) -> Result<bool>;
}
```

### API Reference

#### Functions

##### `from_tree_url(url: &str) -> Result<AndroidFile>`

Creates an `AndroidFile` object from a content tree URI string obtained from the Storage Access Framework (e.g., from an `ACTION_OPEN_DOCUMENT_TREE` intent).

- **Parameters:**
  - `url`: A string slice representing the content tree URI.
- **Returns:**
  - A `Result` containing the `AndroidFile` object if successful, or an error if the URI is invalid or inaccessible.

##### `from_document_file(document_file: &JObject) -> Result<AndroidFile>`

Creates an `AndroidFile` object from a JNI `JObject` that is an instance of `androidx.documentfile.provider.DocumentFile`.

- **Parameters:**
  - `document_file`: A JNI `JObject` reference to a `DocumentFile`.
- **Returns:**
  - A `Result` containing the `AndroidFile` object if successful, or an error if the `JObject` is not a valid `DocumentFile`.

#### `AndroidFileOps` Trait Methods

The following methods are available on `AndroidFile` objects.

##### `open(&self, open_mode: &str) -> Result<File>`

Opens the file represented by the `AndroidFile` object. This method will fail if the `AndroidFile` is a directory.

- **Parameters:**
  - `open_mode`: A string slice specifying the file access mode. The mode corresponds to the modes used in Android's `ContentResolver.openFileDescriptor` method, such as `"r"` (read), `"w"` (write), `"wt"` (write, truncate), `"wa"` (write, append), `"rw"` (read-write), and `"rwt"` (read-write, truncate).
- **Returns:**
  - A `Result` containing a `std::fs::File` object that can be used for reading from or writing to the file, or an error if the file cannot be opened.

##### `list_files(&self) -> Result<Vec<AndroidFile>>`

Lists the files and directories within the directory represented by the `AndroidFile` object. This method will fail if the `AndroidFile` is not a directory.

- **Returns:**
  - A `Result` containing a `Vec<AndroidFile>` with the contents of the directory, or an error if the operation fails.
- **Note:** This operation can be slow due to the underlying Android SAF implementation.

##### `create_file(&self, mime_type: &str, file_name: &str) -> Result<AndroidFile>`

Creates a new file within the directory represented by the `AndroidFile` object. This method will fail if the `AndroidFile` is not a directory.

- **Parameters:**
  - `mime_type`: A string slice representing the MIME type of the new file (e.g., `"text/plain"`).
  - `file_name`: A string slice for the name of the new file.
- **Returns:**
  - A `Result` containing an `AndroidFile` object for the newly created file, or an error if the file cannot be created.

##### `create_directory(&self, dir_name: &str) -> Result<AndroidFile>`

Creates a new directory within the directory represented by the `AndroidFile` object. This method will fail if the `AndroidFile` is not a directory.

- **Parameters:**
  - `dir_name`: A string slice for the name of the new directory.
- **Returns:**
  - A `Result` containing an `AndroidFile` object for the newly created directory, or an error if the directory cannot be created.

##### `remove_file(&self) -> Result<bool>`

Removes the file or directory represented by the `AndroidFile` object. If it's a directory, it will be removed recursively.

- **Returns:**
  - A `Result` containing `true` if the file or directory was successfully deleted, `false` if it did not exist, or an error if the deletion failed.


## Development and Testing

1. Cloning this repository and opening it with Android Studio.
2. Setup NDK and CMake in Android Studio.
3. Specify `ndk.dir` in `local.properties` file.
4. Build and voila!

This project uses [cargo-ndk-android-gradle](https://github.com/willir/cargo-ndk-android-gradle) to integrate Rust code with Android project. The `Cargo.toml` file is located in `app/src/main/rust` directory. You may check that out if anything is not working.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
