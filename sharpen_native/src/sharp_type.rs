use std::sync::Arc;

use crate::{
    TypeId, host_instance::HostInstance, managed_type::GetManagedType, string::CSharpNativeString,
};

pub struct Type {
    id: TypeId,
    base_type: Option<Arc<Type>>,
    element_type: Option<Arc<Type>>,

    host: HostInstance,
}

impl Type {
    pub fn uninit(host: &HostInstance) -> Self {
        Self {
            id: -1,
            base_type: None,
            element_type: None,
            host: host.clone(),
        }
    }

    pub fn from_id(id: TypeId, host: &HostInstance) -> Self {
        Self {
            id,
            base_type: None,
            element_type: None,
            host: host.clone(),
        }
    }

    // TODO: Make these functions safer? return rust types instead of C# types? and probably wrap in Result in case the managed function returns an invalid value?
    pub fn get_full_name(&self) -> String {
        // TODO: Figure out if this leaks memory? does the gc expect us to clean the string up?
        let cs = (self.host.managed_functions().get_full_type_name)(self.id);

        cs.to_string()
    }

    pub fn get_assembly_qualified_name(&self) -> String {
        let cs = (self.host.managed_functions().get_assembly_qualified_name)(self.id);

        cs.to_string()
    }

    pub fn get_base_type(&mut self) -> &Type {
        if self.base_type.is_none() {
            let mut base_type = Type::uninit(&self.host);
            (self.host.managed_functions().get_base_type)(self.id, &mut base_type.id);
            self.base_type = Some(Arc::new(base_type));
        }

        self.base_type.as_ref().unwrap()
    }

    pub fn get_size(&self) -> i32 {
        (self.host.managed_functions().get_type_size)(self.id)
    }

    pub fn is_subclass_of(&self, other: &Self) -> bool {
        // TODO: Figure out the specifics of Bool32 (is true just Bool32 > 0)
        (self.host.managed_functions().is_type_subclass_of)(self.id, other.id) > 0
    }

    pub fn is_assignable_to(&self, other: &Self) -> bool {
        // TODO: Figure out the specifics of Bool32 (is true just Bool32 > 0)
        (self.host.managed_functions().is_type_assignable_to)(self.id, other.id) > 0
    }

    pub fn is_assignable_from(&self, other: &Self) -> bool {
        // TODO: Figure out the specifics of Bool32 (is true just Bool32 > 0)
        (self.host.managed_functions().is_type_assignable_from)(self.id, other.id) > 0
    }

    pub fn get_type_id(&self) -> TypeId {
        self.id
    }
}

pub trait InvokeStaticMethod<Args> {
    // TODO: Does Ret need to be in an Option?
    fn invoke_static_method<Ret>(&self, name: &str, args: Args) -> Ret;
}

impl InvokeStaticMethod<()> for Type {
    fn invoke_static_method<Ret>(&self, name: &str, _args: ()) -> Ret {
        let method_name = CSharpNativeString::new(name);

        let result = unsafe {
            let mut result = std::mem::zeroed::<Ret>();

            (self.host.managed_functions().invoke_static_method_ret)(
                self.id,
                method_name,
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut result as *mut _ as *mut std::ffi::c_void,
            );

            result
        };

        result
    }
}

/* impl<A: 'static> InvokeStaticMethod<(A,)> for Type {
    fn invoke_static_method<Ret>(&self, name: &str, mut args: (A,)) -> Option<Ret> {
        let method_name = CSharpNativeString::new(name);

        let num_params = 1;

        let result = unsafe {
            let parameters = [&mut args.0 as *mut _ as *mut std::ffi::c_void];

            let parameter_types = [A::get_managed_type()];
            println!("{parameter_types:?}");

            let mut result = std::mem::zeroed::<Ret>();

            (self.host.managed_functions().invoke_static_method_ret)(
                self.id,
                method_name,
                &parameters as _,
                &parameter_types as _,
                num_params,
                &mut result as *mut _ as *mut std::ffi::c_void,
            );

            result
        };

        Some(result)
    }
} */

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

macro_rules! impl_invoke_static_method {
	($($idx:tt $arg:tt),+) => {
		impl<$($arg: 'static,)+> InvokeStaticMethod<($($arg,)+)> for Type
		{
			fn invoke_static_method<Ret>(&self, name: &str, mut args: ($($arg,)+)) -> Ret {
				let method_name = CSharpNativeString::new(name);

				let len = count_params!($($arg),+);

				let result = unsafe {
					let parameters = [
						$(&mut args.$idx as *mut _ as *mut std::ffi::c_void),*
					];

					let parameter_types = [
						$($arg::get_managed_type()),*
					];

					let mut result = std::mem::zeroed::<Ret>();

					(self.host.managed_functions().invoke_static_method_ret)(
						self.id,
						method_name,
						&parameters as _,
						&parameter_types as _,
						len,
						&mut result as *mut _ as *mut std::ffi::c_void
					);

					result
				};

				result
			}
		}
	};
}

impl_invoke_static_method!(0 A);
impl_invoke_static_method!(0 A, 1 B);
impl_invoke_static_method!(0 A, 1 B, 2 C);
impl_invoke_static_method!(0 A, 1 B, 2 C, 3 D);
impl_invoke_static_method!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_invoke_static_method!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_invoke_static_method!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
impl_invoke_static_method!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
