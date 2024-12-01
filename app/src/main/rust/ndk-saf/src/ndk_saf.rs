use anyhow::{anyhow, Ok, Result};
use jni::objects::{GlobalRef, JObject, JObjectArray, JString, JValueGen};
use jni::sys::jobject;
use jni::JNIEnv;
use log::info;
use ndk_context::android_context;
use std::fs::File;
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;

// Android File struct definition
#[derive(Debug, Clone)]
pub struct AndroidFile {
    pub filename: String,     // File name
    pub size: usize,          // File size in bytes, behavior undefined for directories
    pub path: String,         // Path (not valid path, only for display)
    pub url: String,          // Content URI (use THIS to obtain the AndroidFile object again)
    pub is_dir: bool,         // Is the file a directory
    document_file: GlobalRef, // JNI DocumentFile JObject representing the file
}

// Android File system features
pub trait AndroidFileOps {
    fn open(&self, open_mode: &str) -> Result<File>;
    fn list_files(&self) -> Result<Vec<AndroidFile>>;
    fn create_file(&self, mime_type: &str, file_name: &str) -> Result<AndroidFile>;
    fn create_directory(&self, dir_name: &str) -> Result<AndroidFile>;
    fn remove_file(&self) -> Result<bool>;
}

fn get_global_context(env: &mut JNIEnv) -> Result<GlobalRef> {
    let activity_thread = env.find_class("android/app/ActivityThread")?;
    let current_activity_thread = env
        .call_static_method(
            activity_thread,
            "currentActivityThread",
            "()Landroid/app/ActivityThread;",
            &[],
        )?
        .l()?;
    let application = env
        .call_method(
            current_activity_thread,
            "getApplication",
            "()Landroid/app/Application;",
            &[],
        )?
        .l()?;
    Ok(env.new_global_ref(application)?)
}

/// Create an AndroidFile object from a content tree URL obtained from Storage Access Framework (SAF).
pub fn from_tree_url(url: &str) -> Result<AndroidFile> {
    info!("Creating AndroidFile object from URL: {}", url);
    // Obtain JNIEnv
    let ctx = android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let context = get_global_context(&mut env)?;

    // Convert Rust string to Java string, and parse it as a URI
    let url_str = env.new_string(url)?;
    let uri = env
        .call_static_method(
            "android/net/Uri",
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValueGen::Object(&url_str)],
        )?
        .l()?;

    // From the documentation of "fromTreeUri", all URIs obtained from OPEN_DOCUMENT_TREE SHOULD ONLY
    // be used with this method. Otherwise, you will get some very unexpected and annoying results.
    let document_file_class = "androidx/documentfile/provider/DocumentFile";
    let document_file = env.call_static_method(
        &document_file_class,
        "fromTreeUri",
        "(Landroid/content/Context;Landroid/net/Uri;)Landroidx/documentfile/provider/DocumentFile;",
        &[JValueGen::Object(context.as_obj()), JValueGen::Object(&uri)],
    )?.l()?;
    Ok(from_document_file(&document_file)?)
}

/// Create an AndroidFile object from a DocumentFile Java object.
pub fn from_document_file(document_file: &JObject) -> Result<AndroidFile> {
    info!(
        "Creating AndroidFile object from DocumentFile object: {:?}",
        document_file
    );
    // First, check if document_file is null
    if document_file.is_null() {
        return Err(anyhow!("The provided DocumentFile object is null"));
    }

    // Obtain JNIEnv
    let ctx = android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;

    // Obtain file name
    let filename = env
        .call_method(document_file, "getName", "()Ljava/lang/String;", &[])?
        .l()
        .and_then(|name| {
            env.get_string(&JString::from(name))
                .map(|s| s.to_string_lossy().into_owned())
        })?;

    // Obtain file size
    let size = env.call_method(&document_file, "length", "()J", &[])?.j()? as usize;

    // Obtain file path and url
    let uri = env
        .call_method(document_file, "getUri", "()Landroid/net/Uri;", &[])?
        .l()?;
    let path_object = env
        .call_method(&uri, "getPath", "()Ljava/lang/String;", &[])?
        .l()?;
    let path = env
        .get_string(&JString::from(path_object))
        .map(|s| s.to_string_lossy().into_owned())?;
    let url = env
        .call_method(&uri, "toString", "()Ljava/lang/String;", &[])?
        .l()
        .and_then(|url| {
            env.get_string(&JString::from(url))
                .map(|s| s.to_string_lossy().into_owned())
        })?;

    // Check if the URL points to a directory
    let is_dir = env
        .call_method(&document_file, "isDirectory", "()Z", &[])?
        .z()
        .unwrap_or(false);

    // Create GlobalRef from DocumentFile object
    let document_file_ref = env.new_global_ref(document_file)?;

    // Construct AndroidFile struct
    Ok(AndroidFile {
        filename,
        size,
        path,
        url,
        is_dir,
        document_file: document_file_ref,
    })
}

impl AndroidFileOps for AndroidFile {
    /// Open the file represented by the AndroidFile object with the specified open mode.
    /// The "open_mode" str corresponds to that in Android ContentResolver.openFileDescriptor method,
    /// which can be "r", "w", "wt", "wa", "rw" or "rwt". Please note that there is <b>no</b> standard
    /// behavior for each mode, and the behavior may vary among different Android versions and file
    /// providers. For example, "w" may truncate the file or not, so it is recommended to specify the
    /// mode explicitly.<br />
    /// Furthermore, "rw" mode requires an on-disk file that supports seeking, while "r" mode and "w"
    /// mode can be used to read or write to a pipe or socket, respectively.
    fn open(&self, open_mode: &str) -> Result<File> {
        // No, you would not want to use this method to open a directory
        if self.is_dir {
            return Err(anyhow!("The provided URL points to a directory"));
        }
        info!("Opening file url: {}, with mode: {}", self.url, open_mode);

        // Obtain JNIEnv and Context
        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let context = get_global_context(&mut env)?;

        // Get ContentResolver object from Context
        let content_resolver = env
            .call_method(
                context,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )?
            .l()?;

        // Convert URI string to Java Uri object, open mode to Java string
        let url_str = env.new_string(&self.url)?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValueGen::Object(&url_str)],
            )?
            .l()?;
        let mode_str = env.new_string(open_mode)?;

        // Open the file descriptor and detach it
        let parcel_fd = env
            .call_method(
                content_resolver,
                "openFileDescriptor",
                "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
                &[JValueGen::Object(&uri), JValueGen::Object(&mode_str)],
            )?
            .l()?;
        let fd = env.call_method(parcel_fd, "detachFd", "()I", &[])?.i()? as RawFd;

        // Create a new file from the file descriptor
        let file = unsafe { File::from_raw_fd(fd) };
        Ok(file)
    }

    /// List files in the directory represented by the AndroidFile object. If the object does not
    /// represent a tree directory, an error will be returned.
    /// THIS IS CURRENTLY VERY SLOW due to the way the Android SAF API is designed. See comments
    /// below for more information.
    fn list_files(&self) -> Result<Vec<AndroidFile>> {
        // Check if the DocumentFile object represents a  directory
        if !self.is_dir {
            return Err(anyhow!("The provided URL does not point to a directory"));
        }
        info!("Listing files in directory: {}", self.url);

        // Obtain JNIEnv
        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast())? };
        let mut env = vm.attach_current_thread()?;

        // Call listFiles method to get the list of files
        // TODO: use this to accelerate the process:
        // https://stackoverflow.com/questions/41096332/issues-traversing-through-directory-hierarchy-with-android-storage-access-framew
        // and why this is so slow:
        // https://stackoverflow.com/questions/42186820/why-is-documentfile-so-slow-and-what-should-i-use-instead
        let files_array = env
            .call_method(
                &self.document_file,
                "listFiles",
                "()[Landroidx/documentfile/provider/DocumentFile;",
                &[],
            )?
            .l()?;
        let files_len = env
            .get_array_length(unsafe { &JObjectArray::from_raw(files_array.clone() as jobject) })?;

        // Iterate through the list of files and get their information
        let mut files = Vec::new();
        for i in 0..files_len {
            let file_obj = env.get_object_array_element(
                unsafe { &JObjectArray::from_raw(files_array.clone() as jobject) },
                i,
            )?;

            // Add the file to the list
            let file = from_document_file(&file_obj)?;
            files.push(file);
        }

        Ok(files)
    }

    /// Create a new file in the directory represented by the AndroidFile object.
    /// If self does not represent a directory, an error will be returned. <br />
    /// PARAMS: MIME type and file name.
    /// The MIME type should be a valid MIME type string, and the file name should not contain any
    /// path separator. When MIME type and extension in file name mismatch, a correct extension will
    /// be appended (thus it is recommended not to include extension).
    /// When names collide, a number will be appended. <br />
    /// RETURNS: A new AndroidFile object representing the newly created file. <br />
    fn create_file(&self, mime_type: &str, file_name: &str) -> Result<AndroidFile> {
        // Check if the DocumentFile object represents a directory
        if !self.is_dir {
            return Err(anyhow!("The provided URL does not point to a directory"));
        }
        info!("Creating file named {} with MIME type {} in directory: {}", file_name, mime_type, self.url);

        // Obtain JNIEnv
        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;

        // Convert MIME type and file name to Java strings
        let mime_type_str = env.new_string(mime_type)?;
        let file_name_str = env.new_string(file_name)?;

        // Create a new file in the directory
        let new_file = env.call_method(
            &self.document_file,
            "createFile",
            "(Ljava/lang/String;Ljava/lang/String;)Landroidx/documentfile/provider/DocumentFile;",
            &[JValueGen::Object(&mime_type_str), JValueGen::Object(&file_name_str)],
        )?.l()?;

        Ok(from_document_file(&new_file)?)
    }

    /// Create a new directory in the directory represented by the AndroidFile object.
    /// If self does not represent a directory, an error will be returned. <br />
    /// PARAMS: Directory name. When names collide, the file name will be appended with a number. <br />
    /// RETURNS: A new AndroidFile object representing the newly created directory. <br />
    fn create_directory(&self, dir_name: &str) -> Result<AndroidFile> {
        // Check if the DocumentFile object represents a directory
        if !self.is_dir {
            return Err(anyhow!("The provided URL does not point to a directory"));
        }
        info!("Creating directory named {} in directory: {}", dir_name, self.url);

        // Obtain JNIEnv
        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;

        // Convert directory name to Java string
        let file_name_str = env.new_string(dir_name)?;

        // Create a new file in the directory
        let new_dir = env
            .call_method(
                &self.document_file,
                "createDirectory",
                "(Ljava/lang/String;)Landroidx/documentfile/provider/DocumentFile;",
                &[JValueGen::Object(&file_name_str)],
            )?
            .l()?;

        Ok(from_document_file(&new_dir)?)
    }

    /// Remove the file or directory represented by the AndroidFile object. If the object represents
    /// a directory, the directory will be removed recursively. The method will return true if the
    /// file or directory is removed successfully, or false if the file or directory does not exist.
    fn remove_file(&self) -> Result<bool> {
        // Obtain JNIEnv
        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;

        // Delete the file or directory
        let result = env
            .call_method(self.document_file.as_obj(), "delete", "()Z", &[])?
            .z()?;

        Ok(result)
    }
}
