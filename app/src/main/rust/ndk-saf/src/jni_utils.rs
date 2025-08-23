use jni::{
    objects::{GlobalRef, JClass, JMethodID},
    AttachGuard, JNIEnv, JavaVM,
};
use log::{error, info};
use std::sync::{Once, RwLock};

use std::sync::atomic::{AtomicPtr, Ordering};

// Thread-safe global state for ClassLoader caching
static INIT: Once = Once::new();
static CLASS_LOADER: RwLock<Option<GlobalRef>> = RwLock::new(None);
static FIND_CLASS_METHOD: RwLock<Option<JMethodID>> = RwLock::new(None);
static JVM: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Initialize the ClassLoader cache with the correct ClassLoader
pub fn initialize_class_loader(
    vm: *mut JavaVM,
    env: &mut JNIEnv,
) -> Result<(), jni::errors::Error> {
    INIT.call_once(|| {
        // Validate JVM pointer
        if vm.is_null() {
            error!("JVM pointer is null");
            return;
        }

        // Store JVM pointer safely
        JVM.store(vm as *mut std::ffi::c_void, Ordering::SeqCst);

        // Setup ClassLoader for proper class finding from non-main threads
        match setup_class_loader(env) {
            Ok((class_loader, find_class_method)) => {
                if let (Ok(mut cl_lock), Ok(mut fcm_lock)) =
                    (CLASS_LOADER.write(), FIND_CLASS_METHOD.write())
                {
                    *cl_lock = Some(class_loader);
                    *fcm_lock = Some(find_class_method);
                    info!("ClassLoader initialized successfully");
                } else {
                    error!("Failed to acquire write locks for ClassLoader initialization");
                }
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
    let _get_class_loader_method =
        env.get_method_id(&class_class, "getClassLoader", "()Ljava/lang/ClassLoader;")?;

    // Get the ClassLoader object
    let class_loader_obj = env.call_method(
        &main_activity_class,
        "getClassLoader",
        "()Ljava/lang/ClassLoader;",
        &[],
    )?;

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

    // Validate VM pointer before casting
    let vm_ptr = ctx.vm() as *const jni::sys::JavaVM;
    if vm_ptr.is_null() {
        return Err(jni::errors::Error::NullPtr("JavaVM pointer is null"));
    }

    // Safe cast to JavaVM
    let vm = unsafe { &*(vm_ptr as *const jni::JavaVM) };
    vm.attach_current_thread()
}

/// Generic class finding function that uses the cached ClassLoader
pub fn find_class(class_name: &str) -> Result<JClass<'_>, jni::errors::Error> {
    let mut env_guard = get_env()?;
    let env = &mut *env_guard;

    // Try to acquire read locks safely
    if let (Ok(class_loader_lock), Ok(find_class_method_lock)) =
        (CLASS_LOADER.read(), FIND_CLASS_METHOD.read())
    {
        match (class_loader_lock.as_ref(), find_class_method_lock.as_ref()) {
            (Some(class_loader), Some(_find_class_method)) => {
                let class_name_jstring = env.new_string(class_name)?;
                let result = env.call_method(
                    class_loader.as_obj(),
                    "findClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[(&class_name_jstring).into()],
                )?;
                Ok(JClass::from(result.l()?))
            }
            _ => {
                // Fallback to standard FindClass if ClassLoader not initialized
                env.find_class(class_name)
            }
        }
    } else {
        // Fallback to standard FindClass if locks cannot be acquired
        env.find_class(class_name)
    }
}

/// Cleanup function for global references (call when library unloads)
pub fn cleanup_class_loader() {
    // Safely acquire write locks and cleanup
    if let Ok(mut class_loader_lock) = CLASS_LOADER.write() {
        if class_loader_lock.take().is_some() {
            // Global references are automatically cleaned up when dropped
        }
    }

    if let Ok(mut find_class_method_lock) = FIND_CLASS_METHOD.write() {
        *find_class_method_lock = None;
    }

    JVM.store(std::ptr::null_mut(), Ordering::SeqCst);

    info!("ClassLoader cleanup completed");
}
