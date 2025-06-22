use std::sync::Arc;

use crate::{
    from_csharp::FromCSharp, host_instance::HostInstance, managed_type::GetManagedType,
    sharp_type::Type, string::CSharpNativeString,
};

pub struct ManagedObject {
    pub(crate) handle: *mut std::ffi::c_void,
    pub(crate) r#type: Option<Arc<Type>>,

    host: HostInstance,
}

impl ManagedObject {
    pub fn uninit(host: &HostInstance) -> Self {
        Self {
            handle: std::ptr::null_mut(),
            r#type: None,
            host: host.clone(),
        }
    }

    pub fn get_type(&mut self) -> Arc<Type> {
        if self.r#type.is_none() {
            let mut r#type = Type::uninit(&self.host);
            (self.host.managed_functions().get_object_type_id)(self.handle, &mut r#type.id as _);

            let r#type = Arc::new(r#type);
            self.host.type_cache().cache_type(r#type.clone());
            self.r#type = Some(r#type);
        }

        self.r#type.clone().unwrap()
    }

    pub fn destroy(self) {
        if self.handle.is_null() {
            return;
        }

        (self.host.managed_functions().destroy_object)(self.handle);
    }

    pub fn is_valid(&self) -> bool {
        !self.handle.is_null() && self.r#type.is_some()
    }

    // TODO: Type conversions
    pub fn set_field_value<FieldType>(&self, name: &str, mut value: FieldType) {
        let mut field_name = CSharpNativeString::new(name);

        (self.host.managed_functions().set_field_value)(
            self.handle,
            field_name.clone(),
            &mut value as *mut FieldType as _,
        );

        CSharpNativeString::free(&mut field_name);
    }

    pub fn get_field_value<CSharp, FieldType: FromCSharp<CSharp>>(&self, name: &str) -> FieldType {
        let mut field_name = CSharpNativeString::new(name);

        let mut result: CSharp = unsafe { std::mem::zeroed() };

        (self.host.managed_functions().get_field_value)(
            self.handle,
            field_name.clone(),
            &mut result as *mut CSharp as _,
        );
        CSharpNativeString::free(&mut field_name);

        FieldType::from_csharp(result)
    }

    pub fn set_property_value<PropertyType>(&self, name: &str, mut value: PropertyType) {
        let mut property_name = CSharpNativeString::new(name);

        (self.host.managed_functions().set_property_value)(
            self.handle,
            property_name.clone(),
            &mut value as *mut PropertyType as _,
        );

        CSharpNativeString::free(&mut property_name);
    }

    pub fn get_property_value<CSharp, PropertyType: FromCSharp<CSharp>>(
        &self,
        name: &str,
    ) -> PropertyType {
        let mut property_name = CSharpNativeString::new(name);

        let mut result: CSharp = unsafe { std::mem::zeroed() };

        // TODO: Safety? How to know if result actually gets set?
        (self.host.managed_functions().get_property_value)(
            self.handle,
            property_name.clone(),
            &mut result as *mut CSharp as _,
        );
        CSharpNativeString::free(&mut property_name);

        PropertyType::from_csharp(result)
    }
}

// TODO: Get/Set Field/Property-value
// TODO: Handle cleanup for CSharpNativeString
pub trait ManagedObjectFns<Args> {
    fn invoke_method<Ret>(&self, name: &str, args: Args) -> Ret;
}

impl ManagedObjectFns<()> for ManagedObject {
    fn invoke_method<Ret>(&self, name: &str, _args: ()) -> Ret {
        let mut method_name = CSharpNativeString::new(name);

        let result = unsafe {
            let mut result = std::mem::zeroed::<Ret>();

            (self.host.managed_functions().invoke_method_ret)(
                self.handle,
                method_name.clone(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut result as *mut _ as *mut std::ffi::c_void,
            );

            result
        };

        CSharpNativeString::free(&mut method_name);

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
				let mut method_name = CSharpNativeString::new(name);

				let len = count_params!($($arg),+);

				let result = unsafe {
					let parameters = [
						$(&mut args.$idx as *mut _ as *mut std::ffi::c_void),*
					];

					let parameter_types = [
						$($arg::get_managed_type()),*
					];

					let mut result = std::mem::zeroed::<Ret>();

					(self.host.managed_functions().invoke_method_ret)(
						self.handle,
						method_name.clone(),
						&parameters as _,
						&parameter_types as _,
						len,
						&mut result as *mut _ as *mut std::ffi::c_void
					);

					result
				};

                CSharpNativeString::free(&mut method_name);

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
