use std::ffi::c_void;

use netcorehost::hostfxr::ManagedFunction;

use crate::{
    Bool32, ManagedHandle, TypeAccessibility, TypeId, managed_type::ManagedType,
    string::CSharpNativeString,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssemblyLoadStatus {
    Success,
    FileNotFound,
    FileLoadFailure,
    InvalidFilePath,
    InvalidAssembly,
    UnknownError,
}

pub type SetInternalCallsFn = extern "system" fn(*mut std::ffi::c_void, i32); // TODO: figure out what *mut c_void is supposed to be
pub type CreateAssemblyLoadContextFn = extern "system" fn(CSharpNativeString) -> i32;
pub type UnloadAssemblyLoadContextFn = extern "system" fn(i32);
pub type LoadAssemblyFn = extern "system" fn(i32, CSharpNativeString) -> i32;
pub type LoadAssemblyFromMemoryFn = extern "system" fn(i32, *const u8, i64) -> i32;
pub type GetLastLoadStatusFn = extern "system" fn() -> AssemblyLoadStatus;
pub type GetAssemblyNameFn = extern "system" fn(i32) -> CSharpNativeString;

pub type GetAssemblyTypesFn = extern "system" fn(i32, *mut TypeId, *mut i32);
pub type GetTypeIdFn = extern "system" fn(CSharpNativeString, *mut TypeId);
pub type GetFullTypeNameFn = extern "system" fn(TypeId) -> CSharpNativeString;
pub type GetAssemblyQualifiedNameFn = extern "system" fn(TypeId) -> CSharpNativeString;
pub type GetBaseTypeFn = extern "system" fn(TypeId, *mut TypeId);
pub type GetTypeSizeFn = extern "system" fn(TypeId) -> i32;
pub type IsTypeSubclassOfFn = extern "system" fn(TypeId, TypeId) -> Bool32;
pub type IsTypeAssignableToFn = extern "system" fn(TypeId, TypeId) -> Bool32;
pub type IsTypeAssignableFromFn = extern "system" fn(TypeId, TypeId) -> Bool32;
pub type IsTypeSZArrayFn = extern "system" fn(TypeId) -> Bool32;
pub type GetElementTypeFn = extern "system" fn(TypeId, *mut TypeId);
pub type GetTypeMethodsFn = extern "system" fn(TypeId, *mut ManagedHandle, *mut i32);
pub type GetTypeFieldsFn = extern "system" fn(TypeId, *mut ManagedHandle, *mut i32);
pub type GetTypePropertiesFn = extern "system" fn(TypeId, *mut ManagedHandle, *mut i32);
pub type HasTypeAttributeFn = extern "system" fn(TypeId, TypeId) -> Bool32;
pub type GetTypeAttributesFn = extern "system" fn(ManagedHandle, *mut TypeId, *mut i32);
pub type GetTypeManagedTypeFn = extern "system" fn(TypeId) -> ManagedType;

pub type GetMethodInfoNameFn = extern "system" fn(ManagedHandle) -> CSharpNativeString;
pub type GetMethodInfoReturnTypeFn = extern "system" fn(ManagedHandle, *mut TypeId);
pub type GetMethodInfoParameterTypesFn = extern "system" fn(ManagedHandle, *mut TypeId, *mut i32);
pub type GetMethodInfoAccessibilityFn = extern "system" fn(ManagedHandle) -> TypeAccessibility;
pub type GetMethodInfoAttributesFn = extern "system" fn(ManagedHandle, *mut TypeId, *mut i32);

pub type GetFieldInfoNameFn = extern "system" fn(ManagedHandle) -> CSharpNativeString;
pub type GetFieldInfoTypeFn = extern "system" fn(ManagedHandle, *mut TypeId);
pub type GetFieldInfoAccessibilityFn = extern "system" fn(ManagedHandle) -> TypeAccessibility;
pub type GetFieldInfoAttributesFn = extern "system" fn(ManagedHandle, *mut TypeId, *mut i32);

pub type GetPropertyInfoNameFn = extern "system" fn(ManagedHandle) -> CSharpNativeString;
pub type GetPropertyInfoTypeFn = extern "system" fn(ManagedHandle, *mut TypeId);
pub type GetPropertyInfoAttributesFn = extern "system" fn(ManagedHandle, *mut TypeId, *mut i32);

pub type GetAttributeFieldValueFn =
    extern "system" fn(ManagedHandle, CSharpNativeString, *mut c_void);
pub type GetAttributeTypeFn = extern "system" fn(ManagedHandle, *mut TypeId);

pub type CreateObjectFn =
    extern "system" fn(TypeId, Bool32, *const *mut c_void, *const ManagedType, i32) -> *mut c_void;
pub type InvokeMethodFn = extern "system" fn(
    *mut c_void,
    CSharpNativeString,
    *const *mut c_void,
    *const ManagedType,
    i32,
);
pub type InvokeMethodRetFn = extern "system" fn(
    *mut c_void,
    CSharpNativeString,
    *const *mut c_void,
    *const ManagedType,
    i32,
    *mut c_void,
);
pub type InvokeStaticMethodFn =
    extern "system" fn(TypeId, CSharpNativeString, *const *mut c_void, *const ManagedType, i32);
pub type InvokeStaticMethodRetFn = extern "system" fn(
    TypeId,
    CSharpNativeString,
    *const *mut c_void,
    *const ManagedType,
    i32,
    *mut c_void,
);

pub type SetFieldValueFn = extern "system" fn(*mut c_void, CSharpNativeString, *mut c_void);
pub type GetFieldValueFn = extern "system" fn(*mut c_void, CSharpNativeString, *mut c_void);
pub type SetPropertyValueFn = extern "system" fn(*mut c_void, CSharpNativeString, *mut c_void);
pub type GetPropertyValueFn = extern "system" fn(*mut c_void, CSharpNativeString, *mut c_void);
pub type DestroyObjectFn = extern "system" fn(*mut c_void);
pub type GetObjectTypeIdFn = extern "system" fn(*mut c_void, *mut i32);

/* pub type CollectGarbageFn = extern "system" fn(i32, GCCollectionMode, Bool32, Bool32);
pub type WaitForPendingFinalizersFn = extern "system" fn(); */

pub struct CoralManagedFunctions {
    pub set_internal_calls: ManagedFunction<SetInternalCallsFn>,
    pub load_assembly: ManagedFunction<LoadAssemblyFn>,
    pub load_assembly_from_memory: ManagedFunction<LoadAssemblyFromMemoryFn>,
    pub unload_assembly_load_context: ManagedFunction<UnloadAssemblyLoadContextFn>,
    pub get_last_load_status: ManagedFunction<GetLastLoadStatusFn>,
    pub get_assembly_name: ManagedFunction<GetAssemblyNameFn>,
    pub get_assembly_types: ManagedFunction<GetAssemblyTypesFn>,
    pub get_type_id: ManagedFunction<GetTypeIdFn>,
    pub get_full_type_name: ManagedFunction<GetFullTypeNameFn>,
    pub get_assembly_qualified_name: ManagedFunction<GetAssemblyQualifiedNameFn>,
    pub get_base_type: ManagedFunction<GetBaseTypeFn>,
    pub get_type_size: ManagedFunction<GetTypeSizeFn>,
    pub is_type_subclass_of: ManagedFunction<IsTypeSubclassOfFn>,
    pub is_type_assignable_to: ManagedFunction<IsTypeAssignableToFn>,
    pub is_type_assignable_from: ManagedFunction<IsTypeAssignableFromFn>,
    pub is_type_sz_array: ManagedFunction<IsTypeSZArrayFn>,
    pub get_element_type: ManagedFunction<GetElementTypeFn>,
    pub get_type_methods: ManagedFunction<GetTypeMethodsFn>,
    pub get_type_fields: ManagedFunction<GetTypeFieldsFn>,
    pub get_type_properties: ManagedFunction<GetTypePropertiesFn>,
    pub has_type_attribute: ManagedFunction<HasTypeAttributeFn>,
    pub get_type_attributes: ManagedFunction<GetTypeAttributesFn>,
    pub get_type_managed_type: ManagedFunction<GetTypeManagedTypeFn>,

    pub get_method_info_name: ManagedFunction<GetMethodInfoNameFn>,
    pub get_method_info_return_type: ManagedFunction<GetMethodInfoReturnTypeFn>,
    pub get_method_info_parameter_types: ManagedFunction<GetMethodInfoParameterTypesFn>,
    pub get_method_info_accessibility: ManagedFunction<GetMethodInfoAccessibilityFn>,
    pub get_method_info_attributes: ManagedFunction<GetMethodInfoAttributesFn>,

    pub get_field_info_name: ManagedFunction<GetFieldInfoNameFn>,
    pub get_field_info_type: ManagedFunction<GetFieldInfoTypeFn>,
    pub get_field_info_accessibility: ManagedFunction<GetFieldInfoAccessibilityFn>,
    pub get_field_info_attributes: ManagedFunction<GetFieldInfoAttributesFn>,

    pub get_property_info_name: ManagedFunction<GetPropertyInfoNameFn>,
    pub get_property_info_type: ManagedFunction<GetPropertyInfoTypeFn>,
    pub get_property_info_attributes: ManagedFunction<GetPropertyInfoAttributesFn>,

    pub get_attribute_field_value: ManagedFunction<GetAttributeFieldValueFn>,
    pub get_attribute_type: ManagedFunction<GetAttributeTypeFn>,

    pub create_object: ManagedFunction<CreateObjectFn>,
    pub create_assembly_load_context: ManagedFunction<CreateAssemblyLoadContextFn>,
    pub invoke_method: ManagedFunction<InvokeMethodFn>,
    pub invoke_method_ret: ManagedFunction<InvokeMethodRetFn>,
    pub invoke_static_method: ManagedFunction<InvokeStaticMethodFn>,
    pub invoke_static_method_ret: ManagedFunction<InvokeStaticMethodRetFn>,

    pub set_field_value: ManagedFunction<SetFieldValueFn>,
    pub get_field_value: ManagedFunction<GetFieldValueFn>,
    pub set_property_value: ManagedFunction<SetPropertyValueFn>,
    pub get_property_value: ManagedFunction<GetPropertyValueFn>,
    pub destroy_object: ManagedFunction<DestroyObjectFn>,
    pub get_object_type_id: ManagedFunction<GetObjectTypeIdFn>,
    /* pub collect_garbage: ManagedFunction<CollectGarbageFn>,
    pub wait_for_pending_finalizers: ManagedFunction<WaitForPendingFinalizersFn>, */
}
