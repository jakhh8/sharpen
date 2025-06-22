pub mod assembly;
pub mod host_instance;
pub mod message_level;
pub mod string;

mod coral_managed_fns;
mod managed_type;
mod sharp_type;
mod type_cache;

pub use sharp_type::InvokeStaticMethod;
pub use type_cache::TypeCacheError;

/// NOTE: Pretty sure this means 32bit
// TODO: Make Bool32 a repr(C, transparent) struct Bool32(c_ulong) to implement a to_bool/from_bool function?
type Bool32 = std::ffi::c_ulong;
type TypeId = std::ffi::c_long;
type ManagedHandle = std::ffi::c_long;

struct InternalCall {
    name: *const netcorehost::pdcstring::PdChar,
    native_function_ptr: *const std::ffi::c_void,
}
