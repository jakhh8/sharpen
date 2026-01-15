pub mod assembly;
pub mod csharp_type;
pub mod from_csharp;
pub mod host_instance;
pub mod interop_types;
pub mod managed_object;
pub mod managed_type;
pub mod message_level;
pub mod type_cache;

mod sharpen_managed_fns;

// TODO: FIGURE OUT WHY THIS IS NEEDED OVER REGULAR BOOL
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct Bool32(pub(crate) std::ffi::c_ulong);
type TypeId = std::ffi::c_long;
type ManagedHandle = std::ffi::c_long;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum TypeAccessibility {
    Public,
    Private,
    Protected,
    Internal,
    ProtectedPublic,
    PrivateProtected,
}

#[allow(unused)]
struct InternalCall {
    name: *const netcorehost::pdcstring::PdChar,
    native_function_ptr: *const std::ffi::c_void,
}

pub type MessageCallbackFn = fn(String, message_level::MessageLevel);
pub(crate) type MessageCallbackFnInternal =
    unsafe extern "system" fn(interop_types::NativeString, message_level::MessageLevel);
pub type ExceptionCallbackFn = fn(String);
pub(crate) type ExceptionCallbackFnInternal =
    unsafe extern "system" fn(interop_types::NativeString);
