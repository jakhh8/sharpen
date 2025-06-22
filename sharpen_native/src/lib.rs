pub mod assembly;
pub mod host_instance;
pub mod message_level;
pub mod meta_info;
pub mod string;

mod coral_managed_fns;
pub mod from_csharp;
pub mod managed_object;
mod managed_type;
mod sharp_type;
mod type_cache;

pub use sharp_type::TypeFns;
pub use type_cache::TypeCacheError;

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
