use std::ffi::c_void;

use netcorehost::{
    hostfxr::{self, ManagedFunction},
    pdcstr, pdcstring,
};

use crate::{
    Bool32, TypeId, assembly::AssemblyLoadStatus, interop_types::NativeString,
    managed_type::ManagedType,
};

pub type CreateAssemblyLoadContextFn = extern "system" fn(NativeString) -> i32;
pub type UnloadAssemblyLoadContextFn = extern "system" fn(i32);
pub type LoadAssemblyFn = extern "system" fn(i32, NativeString) -> i32;
pub type LoadAssemblyFromMemoryFn = extern "system" fn(i32, *const u8, i64) -> i32;
pub type GetLastLoadStatusFn = extern "system" fn() -> AssemblyLoadStatus;
pub type GetAssemblyNameFn = extern "system" fn(i32, i32) -> NativeString;

pub type GetAssemblyTypesFn = extern "system" fn(i32, i32, *mut TypeId, *mut i32);
pub type GetTypeIdFn = extern "system" fn(NativeString, *mut TypeId);
pub type GetFullTypeNameFn = extern "system" fn(TypeId) -> NativeString;
pub type GetAssemblyQualifiedNameFn = extern "system" fn(TypeId) -> NativeString;
pub type GetBaseTypeFn = extern "system" fn(TypeId, *mut TypeId);
pub type GetTypeSizeFn = extern "system" fn(TypeId) -> i32;

pub type CreateObjectFn =
    extern "system" fn(TypeId, Bool32, *const *mut c_void, *const ManagedType, i32) -> *mut c_void;
pub type DestroyObjectFn = extern "system" fn(*mut c_void);

pub type InvokeMethodFn =
    extern "system" fn(*mut c_void, NativeString, *const *mut c_void, *const ManagedType, i32);
pub type InvokeMethodRetFn = extern "system" fn(
    *mut c_void,
    NativeString,
    *const *mut c_void,
    *const ManagedType,
    i32,
    *mut c_void,
);
pub type InvokeStaticMethodFn =
    extern "system" fn(TypeId, NativeString, *const *mut c_void, *const ManagedType, i32);
pub type InvokeStaticMethodRetFn = extern "system" fn(
    TypeId,
    NativeString,
    *const *mut c_void,
    *const ManagedType,
    i32,
    *mut c_void,
);

pub(crate) struct SharpenManagedFunctions {
    pub load_assembly: ManagedFunction<LoadAssemblyFn>,
    pub load_assembly_from_memory: ManagedFunction<LoadAssemblyFromMemoryFn>,
    pub unload_assembly_load_context: ManagedFunction<UnloadAssemblyLoadContextFn>,
    pub get_last_load_status: ManagedFunction<GetLastLoadStatusFn>,
    pub get_assembly_name: ManagedFunction<GetAssemblyNameFn>,

    pub get_assembly_types: ManagedFunction<GetAssemblyTypesFn>,
    pub get_type_id: ManagedFunction<GetTypeIdFn>, // TODO: Consider removing as it's unused
    pub get_full_type_name: ManagedFunction<GetFullTypeNameFn>,
    pub get_assembly_qualified_name: ManagedFunction<GetAssemblyQualifiedNameFn>,
    pub get_base_type: ManagedFunction<GetBaseTypeFn>,
    pub get_type_size: ManagedFunction<GetTypeSizeFn>,

    pub create_object: ManagedFunction<CreateObjectFn>,
    pub destroy_object: ManagedFunction<DestroyObjectFn>,
    pub create_assembly_load_context: ManagedFunction<CreateAssemblyLoadContextFn>,
    pub invoke_method: ManagedFunction<InvokeMethodFn>,
    pub invoke_method_ret: ManagedFunction<InvokeMethodRetFn>,
    pub invoke_static_method: ManagedFunction<InvokeStaticMethodFn>,
    pub invoke_static_method_ret: ManagedFunction<InvokeStaticMethodRetFn>,
}

impl SharpenManagedFunctions {
    pub fn init(
        delegate_loader: &hostfxr::DelegateLoader,
        assembly_path: &pdcstring::PdCStr,
    ) -> Result<Self, hostfxr::GetManagedFunctionError> {
        let create_assembly_load_context = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<CreateAssemblyLoadContextFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.AssemblyLoader, Sharpen.Managed"),
                pdcstr!("CreateAssemblyLoadContext"),
            )?;
        let load_assembly = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<LoadAssemblyFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.AssemblyLoader, Sharpen.Managed"),
                pdcstr!("LoadAssembly"),
            )?;
        let load_assembly_from_memory = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<LoadAssemblyFromMemoryFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.AssemblyLoader, Sharpen.Managed"),
                pdcstr!("LoadAssemblyFromMemory"),
            )?;
        let unload_assembly_load_context = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<UnloadAssemblyLoadContextFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.AssemblyLoader, Sharpen.Managed"),
                pdcstr!("UnloadAssemblyLoadContext"),
            )?;
        let get_last_load_status = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetLastLoadStatusFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.AssemblyLoader, Sharpen.Managed"),
                pdcstr!("GetLastLoadStatus"),
            )?;
        let get_assembly_name = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetAssemblyNameFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.AssemblyLoader, Sharpen.Managed"),
                pdcstr!("GetAssemblyName"),
            )?;

        let get_assembly_types = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetAssemblyTypesFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.TypeInterface, Sharpen.Managed"),
                pdcstr!("GetAssemblyTypes"),
            )?;
        let get_type_id = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeIdFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.TypeInterface, Sharpen.Managed"),
                pdcstr!("GetTypeId"),
            )?;
        let get_full_type_name = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetFullTypeNameFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.TypeInterface, Sharpen.Managed"),
                pdcstr!("GetFullTypeName"),
            )?;
        let get_assembly_qualified_name = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetAssemblyQualifiedNameFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.TypeInterface, Sharpen.Managed"),
                pdcstr!("GetAssemblyQualifiedName"),
            )?;
        let get_base_type = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetBaseTypeFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.TypeInterface, Sharpen.Managed"),
                pdcstr!("GetBaseType"),
            )?;
        let get_type_size = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<GetTypeSizeFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.TypeInterface, Sharpen.Managed"),
                pdcstr!("GetTypeSize"),
            )?;

        let create_object = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<CreateObjectFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.ManagedObject, Sharpen.Managed"),
                pdcstr!("CreateObject"),
            )?;
        let destroy_object = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<DestroyObjectFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.ManagedObject, Sharpen.Managed"),
                pdcstr!("DestroyObject"),
            )?;
        let invoke_method = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeMethodFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.ManagedObject, Sharpen.Managed"),
                pdcstr!("InvokeMethod"),
            )?;
        let invoke_method_ret = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeMethodRetFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.ManagedObject, Sharpen.Managed"),
                pdcstr!("InvokeMethodRet"),
            )?;
        let invoke_static_method = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeStaticMethodFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.ManagedObject, Sharpen.Managed"),
                pdcstr!("InvokeStaticMethod"),
            )?;
        let invoke_static_method_ret = delegate_loader
            .load_assembly_and_get_function_with_unmanaged_callers_only::<InvokeStaticMethodRetFn>(
                assembly_path,
                pdcstr!("Sharpen.Managed.ManagedObject, Sharpen.Managed"),
                pdcstr!("InvokeStaticMethodRet"),
            )?;

        Ok(SharpenManagedFunctions {
            create_assembly_load_context,
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

            create_object,
            destroy_object,
            invoke_method,
            invoke_method_ret,
            invoke_static_method,
            invoke_static_method_ret,
        })
    }
}
