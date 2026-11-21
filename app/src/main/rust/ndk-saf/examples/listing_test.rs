use jni::{
    objects::{JClass, JString},
    sys::jint,
    JNIEnv,
};
use ndk_saf::{AndroidFile, AndroidFileOps};

#[no_mangle]
pub extern "system" fn Java_one_rachelt_rust_1saf_ListingTest_count(
    mut env: JNIEnv,
    _: JClass,
    uri: JString,
) -> jint {
    let result = (|| -> anyhow::Result<_> {
        let vm = env.get_java_vm()?;
        ndk_saf::initialize_class_loader(vm.get_java_vm_pointer().cast(), &mut env)?;
        let dir = AndroidFile {
            filename: String::new(),
            size: 0,
            path: String::new(),
            url: env.get_string(&uri)?.into(),
            is_dir: true,
        };
        Ok(dir.list_files()?.len() as jint)
    })();
    result.unwrap_or_else(|error| {
        env.throw_new("java/io/IOException", error.to_string())
            .unwrap();
        -1
    })
}
