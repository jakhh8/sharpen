use std::sync::{Arc, Mutex, MutexGuard};

use netcorehost::{hostfxr, nethost, pdcstr, pdcstring};

use crate::{
    assembly::AssemblyLoadContext,
    coral_managed_fns::*,
    message_level::{MessageCallbackFn, MessageCallbackFnInternal, MessageLevel},
    string::{CSharpNativeString, ScopedCSharpNativeString},
    type_cache::TypeCache,
};

#[derive(Debug, Clone, Copy)]
pub enum CoralInitError {
    FailedToLoadHostFXR,
    CoralManagedNotFound,
    CoralManagedInitError(CoralManagedInitError),
}

#[derive(Debug, Clone, Copy)]
pub enum CoralManagedInitError {
    CouldNotInitializeForRuntimeConfig,
    FailedToGetDelegateLoader,
    CouldNotLoadFnPtr,
    CouldNotLoadCoralFunctions,
}

pub type ExceptionCallbackFn = fn(String);
pub(crate) type ExceptionCallbackFnInternal = fn(CSharpNativeString);

#[derive(Clone)]
pub struct HostSettings {
    /// The file path to Coral.runtimeconfig.json (e.g C:\Dev\MyProject\ThirdParty\Coral)
    pub coral_directory: std::path::PathBuf,

    pub message_callback: Option<MessageCallbackFn>,
    pub messsage_filter: MessageLevel,

    pub exception_callback: Option<ExceptionCallbackFn>,
}

#[derive(Clone)]
pub struct HostInstance {
    settings: HostSettings,
    coral_managed_assembly_path: std::path::PathBuf,

    managed_functions: Arc<CoralManagedFunctions>,
    type_cache: Arc<Mutex<TypeCache>>,
}

impl HostInstance {
    pub fn initialize(settings: HostSettings) -> Result<Self, CoralInitError> {
        let hostfxr = nethost::load_hostfxr().map_err(|_| CoralInitError::FailedToLoadHostFXR)?;

        // TODO: Fix unsafe
        unsafe {
            MESSAGE_CALLBACK = settings
                .message_callback
                .unwrap_or(default_message_callback);
            MESSAGE_FILTER = settings.messsage_filter;
            EXCEPTION_CALLBACK = settings.exception_callback;
        }

        let coral_managed_assembly_path = settings.coral_directory.join("Coral.Managed.dll");
        if !coral_managed_assembly_path.exists() {
            message_callback(
                CSharpNativeString::new("Failed to find Coral.Managed.dll"),
                MessageLevel::Error,
            );
            return Err(CoralInitError::CoralManagedNotFound);
        }

        let managed_functions = Arc::new(
            Self::initialize_coral_managed(&hostfxr, &settings, &coral_managed_assembly_path)
                .map_err(|err| CoralInitError::CoralManagedInitError(err))?,
        );

        Ok(Self {
            settings,
            coral_managed_assembly_path,

            managed_functions,
            type_cache: Arc::new(Mutex::new(TypeCache::new())),
        })
    }

    pub fn create_assembly_load_context(&self, name: &str) -> AssemblyLoadContext {
        let name = ScopedCSharpNativeString::from_str(name);

        AssemblyLoadContext::new(
            (self.managed_functions.create_assembly_load_context)(name.inner()),
            self,
        )
    }

    pub fn type_cache(&self) -> MutexGuard<TypeCache> {
        self.type_cache.lock().expect("TypeCache Mutex is poisoned")
    }

    /// Automatically called when AssemblyLoadContext is dropped
    pub(crate) fn unload_assembly_load_context(&self, assembly_load_context: &AssemblyLoadContext) {
        (self.managed_functions.unload_assembly_load_context)(assembly_load_context.context_id());
    }

    pub(crate) fn managed_functions(&self) -> &CoralManagedFunctions {
        &self.managed_functions
    }
}

// TODO: Fix this bull
static mut MESSAGE_CALLBACK: MessageCallbackFn = |_, _| {
    panic!("MESSAGE_CALLBACK called before being initialized");
};
static mut MESSAGE_FILTER: MessageLevel = MessageLevel::Info;
static mut EXCEPTION_CALLBACK: Option<ExceptionCallbackFn> = None;

#[inline]
extern "system" fn message_callback(in_message: CSharpNativeString, in_level: MessageLevel) {
    let message = in_message.to_string();
    unsafe {
        MESSAGE_CALLBACK(message, in_level);
    }
}

#[inline]
extern "system" fn exception_callback(in_message: CSharpNativeString) {
    let message = in_message.to_string();
    unsafe {
        EXCEPTION_CALLBACK.map_or_else(
            || {
                message_callback(in_message, MessageLevel::Error);
                return;
            },
            |exception_callback| {
                exception_callback(message);
            },
        );
    }
}

impl HostInstance {
    fn initialize_coral_managed(
        hostfxr: &hostfxr::Hostfxr,
        settings: &HostSettings,
        coral_managed_assembly_path: &std::path::Path,
    ) -> Result<CoralManagedFunctions, CoralManagedInitError> {
        let runtime_config_path = settings
            .coral_directory
            .join("Coral.Managed.runtimeconfig.json");
        let runtime_config_path_pdcstr =
            pdcstring::PdCString::from_os_str(runtime_config_path.as_os_str())
                .expect("Failed to generate PdCString!");

        let context = hostfxr
            .initialize_for_runtime_config(&runtime_config_path_pdcstr)
            .map_err(|_| CoralManagedInitError::CouldNotInitializeForRuntimeConfig)?;
        let delegate_loader = context
            .get_delegate_loader()
            .map_err(|_| CoralManagedInitError::FailedToGetDelegateLoader)?;

        let coral_managed_assembly_path_pdcstr =
            pdcstring::PdCString::from_os_str(coral_managed_assembly_path.as_os_str())
                .expect("wtf!");

        type InitializeFn = extern "system" fn(
            extern "system" fn(CSharpNativeString, MessageLevel),
            extern "system" fn(CSharpNativeString),
        );
        let coral_managed_entrypoint = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InitializeFn>(
                &coral_managed_assembly_path_pdcstr,
                pdcstr!("Coral.Managed.ManagedHost, Coral.Managed"),
                pdcstr!("Initialize"),
            )
            .map_err(|_| CoralManagedInitError::CouldNotLoadFnPtr)?;

        let managed_functions =
            Self::load_coral_functions(&delegate_loader, &coral_managed_assembly_path_pdcstr)
                .map_err(|_| CoralManagedInitError::CouldNotLoadCoralFunctions)?;

        coral_managed_entrypoint(message_callback, exception_callback);

        Ok(managed_functions)
    }

    fn load_coral_functions(
        delegate_loader: &hostfxr::DelegateLoader,
        assembly_path: &pdcstring::PdCStr,
    ) -> Result<CoralManagedFunctions, hostfxr::GetManagedFunctionError> {
        let create_assembly_load_context = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<CreateAssemblyLoadContextFn>(
                assembly_path,
                pdcstr!("Coral.Managed.AssemblyLoader, Coral.Managed"),
                pdcstr!("CreateAssemblyLoadContext"),
            )?;

        let set_internal_calls = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<SetInternalCallsFn>(
                assembly_path,
                pdcstr!("Coral.Managed.Interop.InternalCallsManager, Coral.Managed"),
                pdcstr!("SetInternalCalls"),
            )?;
        let load_assembly = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<LoadAssemblyFn>(
                assembly_path,
                pdcstr!("Coral.Managed.AssemblyLoader, Coral.Managed"),
                pdcstr!("LoadAssembly"),
            )?;
        let load_assembly_from_memory = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<LoadAssemblyFromMemoryFn>(
                assembly_path,
                pdcstr!("Coral.Managed.AssemblyLoader, Coral.Managed"),
                pdcstr!("LoadAssemblyFromMemory"),
            )?;
        let unload_assembly_load_context = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<UnloadAssemblyLoadContextFn>(
                assembly_path,
                pdcstr!("Coral.Managed.AssemblyLoader, Coral.Managed"),
                pdcstr!("UnloadAssemblyLoadContext"),
            )?;
        let get_last_load_status = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetLastLoadStatusFn>(
                assembly_path,
                pdcstr!("Coral.Managed.AssemblyLoader, Coral.Managed"),
                pdcstr!("GetLastLoadStatus"),
            )?;
        let get_assembly_name = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetAssemblyNameFn>(
                assembly_path,
                pdcstr!("Coral.Managed.AssemblyLoader, Coral.Managed"),
                pdcstr!("GetAssemblyName"),
            )?;

        let get_assembly_types = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetAssemblyTypesFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetAssemblyTypes"),
            )?;
        let get_type_id = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeIdFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetTypeId"),
            )?;
        let get_full_type_name = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetFullTypeNameFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetFullTypeName"),
            )?;
        let get_assembly_qualified_name = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetAssemblyQualifiedNameFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetAssemblyQualifiedName"),
            )?;
        let get_base_type = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetBaseTypeFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetBaseType"),
            )?;
        let get_type_size = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeSizeFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetTypeSize"),
            )?;
        let is_type_subclass_of = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<IsTypeSubclassOfFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("IsTypeSubclassOf"),
            )?;
        let is_type_assignable_to = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<IsTypeAssignableToFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("IsTypeAssignableTo"),
            )?;
        let is_type_assignable_from = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<IsTypeAssignableFromFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("IsTypeAssignableFrom"),
            )?;
        let is_type_sz_array = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<IsTypeSZArrayFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("IsTypeSZArray"),
            )?;
        let get_element_type = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetElementTypeFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetElementType"),
            )?;
        let get_type_methods = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeMethodsFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetTypeMethods"),
            )?;
        let get_type_fields = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeFieldsFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetTypeFields"),
            )?;
        let get_type_properties = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypePropertiesFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetTypeProperties"),
            )?;
        let has_type_attribute = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<HasTypeAttributeFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("HasTypeAttribute"),
            )?;
        let get_type_attributes = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeAttributesFn>(
                assembly_path,
                pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
                pdcstr!("GetTypeAttributes"),
            )?;
        /* let get_type_managed_type = delegate_loader
        .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeManagedTypeFn>(
            assembly_path,
            pdcstr!("Coral.Managed.TypeInterface, Coral.Managed"),
            pdcstr!("GetTypeManagedType"),
        )?; */

        let invoke_method = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeMethodFn>(
                assembly_path,
                pdcstr!("Coral.Managed.ManagedObject, Coral.Managed"),
                pdcstr!("InvokeMethod"),
            )?;
        let invoke_method_ret = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeMethodRetFn>(
                assembly_path,
                pdcstr!("Coral.Managed.ManagedObject, Coral.Managed"),
                pdcstr!("InvokeMethodRet"),
            )?;
        let invoke_static_method = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeStaticMethodFn>(
                assembly_path,
                pdcstr!("Coral.Managed.ManagedObject, Coral.Managed"),
                pdcstr!("InvokeStaticMethod"),
            )?;
        let invoke_static_method_ret = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeStaticMethodRetFn>(
                assembly_path,
                pdcstr!("Coral.Managed.ManagedObject, Coral.Managed"),
                pdcstr!("InvokeStaticMethodRet"),
            )?;

        Ok(CoralManagedFunctions {
            create_assembly_load_context,

            set_internal_calls,
            load_assembly,
            load_assembly_from_memory,
            unload_assembly_load_context,
            get_last_load_status,
            get_assembly_name,

            get_assembly_types,
            get_type_id,
            get_full_type_name,
            get_assembly_qualified_name,
            get_base_type,
            get_type_size,
            is_type_subclass_of,
            is_type_assignable_to,
            is_type_assignable_from,
            is_type_sz_array,
            get_element_type,
            get_type_methods,
            get_type_fields,
            get_type_properties,
            has_type_attribute,
            get_type_attributes,
            //get_type_managed_type,
            invoke_method,
            invoke_method_ret,
            invoke_static_method,
            invoke_static_method_ret,
        })
    }
}

fn default_message_callback(message: String, level: MessageLevel) {
    println!("[Sharpen]({level}): {message}");
}
