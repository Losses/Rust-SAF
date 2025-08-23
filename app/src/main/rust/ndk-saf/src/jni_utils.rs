use std::sync::Once;
use jni::{
    objects::{GlobalRef, JClass, JMethodID},
    sys::jvalue,
    AttachGuard, JNIEnv, JavaVM,
};
use log::{error, info};

// Global state for ClassLoader caching
static INIT: Once = Once::new();
static mut CLASS_LOADER: Option<GlobalRef> = None;
static mut FIND_CLASS_METHOD: Option<JMethodID> = None;
static mut JVM: Option<*mut std::ffi::c_void> = None;

/// Initialize the ClassLoader cache with the correct ClassLoader
pub fn initialize_class_loader(vm: *mut JavaVM, env: &mut JNIEnv) -> Result<(), jni::errors::Error> {
    INIT.call_once(|| unsafe {
        // Store JVM pointer
        JVM = Some(vm as *mut std::ffi::c_void);
        
        // Setup ClassLoader for proper class finding from non-main threads
        match setup_class_loader(env) {
            Ok((class_loader, find_class_method)) => {
                CLASS_LOADER = Some(class_loader);
                FIND_CLASS_METHOD = Some(find_class_method);
                info!("ClassLoader initialized successfully");
            }
            Err(e) => {
                error!("Failed to setup ClassLoader: {:?}", e);
            }
        }
    });
    Ok(())
}

/// Setup ClassLoader during initialization to cache for later use
fn setup_class_loader(env: &mut JNIEnv) -> Result<(GlobalRef, JMethodID), jni::errors::Error> {
    // Use MainActivity as our reference class to get the correct ClassLoader
    let main_activity_class = env.find_class("one/rachelt/rust_saf/MainActivity")?;
    let class_class = env.get_object_class(&main_activity_class)?;
    let class_loader_class = env.find_class("java/lang/ClassLoader")?;
    
    // Get the getClassLoader method
    let get_class_loader_method = env.get_method_id(
        &class_class,
        "getClassLoader",
        "()Ljava/lang/ClassLoader;",
    )?;
    
    // Get the ClassLoader object
    let class_loader_obj = unsafe {
        env.call_method_unchecked(
            &main_activity_class,
            get_class_loader_method,
            jni::signature::ReturnType::Object,
            &[],
        )?
    };
    
    let class_loader = env.new_global_ref(class_loader_obj.l()?)?;
    
    // Cache the findClass method ID
    let find_class_method = env.get_method_id(
        &class_loader_class,
        "findClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
    )?;
    
    Ok((class_loader, find_class_method))
}

/// Improved getEnv function that handles thread attachment properly
pub fn get_env() -> Result<AttachGuard<'static>, jni::errors::Error> {
    use ndk_context::android_context;
    let ctx = android_context();
    // Use the VM from the android context directly
    let vm = unsafe { &*(ctx.vm() as *const jni::sys::JavaVM as *const jni::JavaVM) };
    vm.attach_current_thread()
}

/// Generic class finding function that uses the cached ClassLoader
pub fn find_class(class_name: &str) -> Result<JClass<'_>, jni::errors::Error> {
    let mut env_guard = get_env()?;
    let env = &mut *env_guard;
    
    unsafe {
        let class_loader_ptr = &raw const CLASS_LOADER;
        let find_class_method_ptr = &raw const FIND_CLASS_METHOD;
        match ((*class_loader_ptr).as_ref(), (*find_class_method_ptr).as_ref()) {
            (Some(class_loader), Some(find_class_method)) => {
                let class_name_jstring = env.new_string(class_name)?;
                let result = env.call_method_unchecked(
                    class_loader.as_obj(),
                    *find_class_method,
                    jni::signature::ReturnType::Object,
                    &[jvalue { l: class_name_jstring.as_raw() }],
                )?;
                Ok(JClass::from(result.l()?))
            }
            _ => {
                // Fallback to standard FindClass if ClassLoader not initialized
                env.find_class(class_name)
            }
        }
    }
}

/// Cleanup function for global references (call when library unloads)
pub fn cleanup_class_loader() {
    unsafe {
        let class_loader_ptr = &raw mut CLASS_LOADER;
        if let Some(_class_loader) = (*class_loader_ptr).take() {
            // Global references are automatically cleaned up when dropped
        }
        let find_class_method_ptr = &raw mut FIND_CLASS_METHOD;
        *find_class_method_ptr = None;
        let jvm_ptr = &raw mut JVM;
        *jvm_ptr = None;
        info!("ClassLoader cleanup completed");
    }
}