use std::sync::Arc;

use crate::{
    csharp_type::Type, interop_types::NativeString, managed_type::GetManagedType,
    sharpen_managed_fns::SharpenManagedFunctions,
};

pub struct ManagedObject {
    pub(crate) handle: *mut std::ffi::c_void,
    // TODO: Does this need to know its own type at all
    pub(crate) r#type: Option<Arc<Type>>,

    managed_funcs: Arc<SharpenManagedFunctions>,
}

impl ManagedObject {
    pub(crate) fn uninit(managed_funcs: Arc<SharpenManagedFunctions>) -> Self {
        Self {
            handle: std::ptr::null_mut(),
            r#type: None,
            managed_funcs,
        }
    }

    pub fn destroy(self) {
        if self.handle.is_null() {
            return;
        }

        (self.managed_funcs.destroy_object)(self.handle);
    }
}

// TODO: Handle cleanup for CSharpNativeString
pub trait ManagedObjectFns<Args> {
    fn invoke_method<Ret>(&self, name: &str, args: Args) -> Ret;
}

impl ManagedObjectFns<()> for ManagedObject {
    fn invoke_method<Ret>(&self, name: &str, _args: ()) -> Ret {
        let mut method_name = NativeString::new(name);

        let result = unsafe {
            let mut result = std::mem::zeroed::<Ret>();

            (self.managed_funcs.invoke_method_ret)(
                self.handle,
                method_name.clone(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut result as *mut _ as *mut std::ffi::c_void,
            );

            result
        };

        NativeString::free(&mut method_name);

        result
    }
}

macro_rules! count_params {
	($first:tt $(, $rest:tt)*) => {
		count_params!(0 ; $first $(, $rest)*)
	};
	($count:expr ; $first:tt $(, $rest:tt)*) => {
		count_params!($count + 1 ; $($rest),*)
	};
	($count:expr ;) => {
		$count
	};
}

macro_rules! impl_type_fns {
	($($idx:tt $arg:tt),+) => {
		impl<$($arg: 'static,)+> ManagedObjectFns<($($arg,)+)> for ManagedObject
		{
			fn invoke_method<Ret>(&self, name: &str, mut args: ($($arg,)+)) -> Ret {
				let mut method_name = NativeString::new(name);

				let len = count_params!($($arg),+);

				let result = unsafe {
					let parameters = [
						$(&mut args.$idx as *mut _ as *mut std::ffi::c_void),*
					];

					let parameter_types = [
						$($arg::get_managed_type()),*
					];

					let mut result = std::mem::zeroed::<Ret>();

					(self.managed_funcs.invoke_method_ret)(
						self.handle,
						method_name.clone(),
						&parameters as _,
						&parameter_types as _,
						len,
						&mut result as *mut _ as *mut std::ffi::c_void
					);

					result
				};

                NativeString::free(&mut method_name);

				result
			}
		}
	};
}

impl_type_fns!(0 A);
impl_type_fns!(0 A, 1 B);
impl_type_fns!(0 A, 1 B, 2 C);
impl_type_fns!(0 A, 1 B, 2 C, 3 D);
impl_type_fns!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_type_fns!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_type_fns!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
impl_type_fns!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
