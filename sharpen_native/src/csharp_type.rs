use std::sync::{Arc, Mutex};

use crate::{
    TypeId, interop_types::NativeString, managed_object::ManagedObject,
    managed_type::GetManagedType, sharpen_managed_fns::SharpenManagedFunctions,
    type_cache::TypeCache,
};

pub struct Type {
    pub(crate) id: TypeId,
    base_type: Option<Arc<Type>>,

    managed_funcs: Arc<SharpenManagedFunctions>,
    type_cache: Arc<Mutex<TypeCache>>,
}

impl Type {
    pub(crate) fn uninit(
        managed_funcs: Arc<SharpenManagedFunctions>,
        type_cache: Arc<Mutex<TypeCache>>,
    ) -> Self {
        Self {
            id: -1,
            base_type: None,

            managed_funcs,
            type_cache,
        }
    }

    pub(crate) fn from_id(
        id: TypeId,
        managed_funcs: Arc<SharpenManagedFunctions>,
        type_cache: Arc<Mutex<TypeCache>>,
    ) -> Self {
        Self {
            id,
            base_type: None,

            managed_funcs,
            type_cache,
        }
    }

    // TODO: Make these functions safer? return rust types instead of C# types? and probably wrap in Result in case the managed function returns an invalid value?
    pub fn get_full_name(&self) -> String {
        // TODO: Figure out if this leaks memory? does the gc expect us to clean the string up?
        let cs = (self.managed_funcs.get_full_type_name)(self.id);

        cs.to_string()
    }

    pub fn get_assembly_qualified_name(&self) -> String {
        let cs = (self.managed_funcs.get_assembly_qualified_name)(self.id);

        cs.to_string()
    }

    pub fn get_base_type(&mut self) -> &Type {
        if self.base_type.is_none() {
            let mut base_type = Type::uninit(self.managed_funcs.clone(), self.type_cache.clone());
            (self.managed_funcs.get_base_type)(self.id, &mut base_type.id);
            self.base_type = Some(Arc::new(base_type));
        }

        self.base_type.as_ref().unwrap()
    }

    pub fn get_size(&self) -> i32 {
        (self.managed_funcs.get_type_size)(self.id)
    }

    pub fn get_type_id(&self) -> TypeId {
        self.id
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Type {}

pub trait TypeFns<Args> {
    fn create_instance(&self, args: Args) -> ManagedObject;
    fn invoke_static_method<Ret>(&self, name: &str, args: Args) -> Ret;
}

impl TypeFns<()> for Type {
    fn create_instance(&self, _args: ()) -> ManagedObject {
        let mut object = ManagedObject::uninit(self.managed_funcs.clone());
        object.handle = (self.managed_funcs.create_object)(
            self.id,
            false.into(),
            std::ptr::null_mut(),
            std::ptr::null(),
            0,
        );
        // TODO: Is this the best way to do this?
        object.r#type = Some(
            self.type_cache
                .lock()
                .unwrap()
                .get_type_by_id(self.id)
                .unwrap(),
        );

        object
    }

    fn invoke_static_method<Ret>(&self, name: &str, _args: ()) -> Ret {
        let mut method_name = NativeString::new(name);

        let result = unsafe {
            let mut result = std::mem::zeroed::<Ret>();

            (self.managed_funcs.invoke_static_method_ret)(
                self.id,
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

// TODO: Rewrite impl_type_fns to single out the last idx/arg pair, so this macro is no longer needed
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
		impl<$($arg: 'static,)+> TypeFns<($($arg,)+)> for Type
		{
            fn create_instance(&self, mut args: ($($arg,)+)) -> ManagedObject {
                let mut object = ManagedObject::uninit(self.managed_funcs.clone());

				let len = count_params!($($arg),+);

                let parameters = [
                    $(&mut args.$idx as *mut _ as *mut std::ffi::c_void),*
                ];

                let parameter_types = [
                    $($arg::get_managed_type()),*
                ];

                object.handle = (self.managed_funcs.create_object)(
                    self.id,
                    false.into(),
                    &parameters as _,
                    &parameter_types as _,
                    len,
                );
                // TODO: Is this the best way to do this?
                object.r#type = Some(self.type_cache
                .lock()
                .unwrap().get_type_by_id(self.id).unwrap());

                object
            }

			fn invoke_static_method<Ret>(&self, name: &str, mut args: ($($arg,)+)) -> Ret {
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

					(self.managed_funcs.invoke_static_method_ret)(
						self.id,
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
