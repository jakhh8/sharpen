use std::sync::{Arc, Mutex, OnceLock};

use netcorehost::{hostfxr, nethost, pdcstr, pdcstring};

use crate::{
    ExceptionCallbackFn, ExceptionCallbackFnInternal, MessageCallbackFn, MessageCallbackFnInternal,
    assembly::AssemblyLoadContext,
    interop_types::{NativeString, ScopedNativeString},
    message_level::MessageLevel,
    sharpen_managed_fns::*,
    type_cache::TypeCache,
};

#[derive(Debug, Clone, Copy)]
pub enum SharpenInitError {
    FailedToLoadHostFXR,
    SharpenManagedNotFound,
    SharpenManagedInitError(SharpenManagedInitError),
}

#[derive(Debug, Clone, Copy)]
pub enum SharpenManagedInitError {
    CouldNotInitializeForRuntimeConfig,
    FailedToGetDelegateLoader,
    CouldNotLoadFnPtr,
    CouldNotLoadSharpenFunctions,
}

#[derive(Clone)]
pub struct HostSettings {
    /// The file path to Sharpen.runtimeconfig.json (e.g C:\Dev\MyProject\ThirdParty\Sharpen)
    pub sharpen_directory: std::path::PathBuf,

    pub message_callback: Option<MessageCallbackFn>,
    pub messsage_filter: MessageLevel,

    pub exception_callback: Option<ExceptionCallbackFn>,
}

#[derive(Clone)]
pub struct HostInstance {
    managed_functions: Arc<SharpenManagedFunctions>,
    // TODO: Should TypeCache be local to the assembly maybe?
    type_cache: Arc<Mutex<TypeCache>>,
}

impl HostInstance {
    pub fn initialize(settings: HostSettings) -> Result<Self, SharpenInitError> {
        let hostfxr = nethost::load_hostfxr().map_err(|_| SharpenInitError::FailedToLoadHostFXR)?;

        MESSAGE_CALLBACK
            .set(
                settings
                    .message_callback
                    .unwrap_or(default_message_callback),
            )
            .expect("MessageCallback already set!");
        MESSAGE_FILTER
            .set(settings.messsage_filter)
            .expect("MessageFilter already set!");
        if let Some(callback) = settings.exception_callback {
            EXCEPTION_CALLBACK
                .set(callback)
                .expect("ExceptionCallback already set!");
        }

        let sharpen_managed_assembly_path = settings.sharpen_directory.join("Sharpen.Managed.dll");
        if !sharpen_managed_assembly_path.exists() {
            message_callback(
                NativeString::new("Failed to find Sharpen.Managed.dll"),
                MessageLevel::Error,
            );
            return Err(SharpenInitError::SharpenManagedNotFound);
        }

        let managed_functions = Arc::new(
            Self::initialize_sharpen_managed(&hostfxr, &settings, &sharpen_managed_assembly_path)
                .map_err(|err| SharpenInitError::SharpenManagedInitError(err))?,
        );

        Ok(Self {
            managed_functions,
            type_cache: Arc::new(Mutex::new(TypeCache::new())),
        })
    }

    fn initialize_sharpen_managed(
        hostfxr: &hostfxr::Hostfxr,
        settings: &HostSettings,
        sharpen_managed_assembly_path: &std::path::Path,
    ) -> Result<SharpenManagedFunctions, SharpenManagedInitError> {
        let runtime_config_path = settings
            .sharpen_directory
            .join("Sharpen.Managed.runtimeconfig.json");
        let runtime_config_path_pdcstr =
            pdcstring::PdCString::from_os_str(runtime_config_path.as_os_str())
                .expect("Failed to generate PdCString!");

        let context = hostfxr
            .initialize_for_runtime_config(&runtime_config_path_pdcstr)
            .map_err(|_| SharpenManagedInitError::CouldNotInitializeForRuntimeConfig)?;
        let delegate_loader = context
            .get_delegate_loader()
            .map_err(|_| SharpenManagedInitError::FailedToGetDelegateLoader)?;

        let sharpen_managed_assembly_path_pdcstr =
            pdcstring::PdCString::from_os_str(sharpen_managed_assembly_path.as_os_str())
                .expect("wtf!");

        type InitializeFn =
            extern "system" fn(MessageCallbackFnInternal, ExceptionCallbackFnInternal);
        let sharpen_managed_entrypoint = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InitializeFn>(
                &sharpen_managed_assembly_path_pdcstr,
                pdcstr!("Sharpen.Managed.ManagedHost, Sharpen.Managed"),
                pdcstr!("Initialize"),
            )
            .map_err(|_| SharpenManagedInitError::CouldNotLoadFnPtr)?;

        let managed_functions =
            SharpenManagedFunctions::init(&delegate_loader, &sharpen_managed_assembly_path_pdcstr)
                .map_err(|_| SharpenManagedInitError::CouldNotLoadSharpenFunctions)?;

        sharpen_managed_entrypoint(message_callback, exception_callback);

        Ok(managed_functions)
    }

    pub fn create_assembly_load_context(&self, name: &str) -> AssemblyLoadContext {
        let name = ScopedNativeString::from_str(name);

        AssemblyLoadContext::new(
            (self.managed_functions.create_assembly_load_context)(name.inner()),
            self.managed_functions.clone(),
            self.type_cache.clone(),
        )
    }
}

// TODO: Fix this bull probably with OnceLock
static MESSAGE_CALLBACK: OnceLock<MessageCallbackFn> = OnceLock::new();
static MESSAGE_FILTER: OnceLock<MessageLevel> = OnceLock::new();
static EXCEPTION_CALLBACK: OnceLock<ExceptionCallbackFn> = OnceLock::new();

#[inline]
extern "system" fn message_callback(in_message: NativeString, in_level: MessageLevel) {
    let message = in_message.to_string();

    (MESSAGE_CALLBACK
        .get()
        .expect("MessageCallback called before being set"))(message, in_level);
}

#[inline]
extern "system" fn exception_callback(in_message: NativeString) {
    let message = in_message.to_string();

    EXCEPTION_CALLBACK.get().map_or_else(
        || {
            message_callback(in_message, MessageLevel::Error);
        },
        |exception_callback| {
            exception_callback(message);
        },
    );
}

fn default_message_callback(message: String, level: MessageLevel) {
    println!("[Sharpen]({level}): {message}");
}
