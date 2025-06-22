use std::sync::Arc;

use crate::{
    TypeId,
    host_instance::HostInstance,
    managed_object::ManagedObject,
    managed_type::{GetManagedType, ManagedType},
    meta_info::{Attribute, FieldInfo, MethodInfo, PropertyInfo},
    string::CSharpNativeString,
};

pub struct Type {
    pub(crate) id: TypeId,
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
        (self.host.managed_functions().is_type_subclass_of)(self.id, other.id).into()
    }

    pub fn is_assignable_to(&self, other: &Self) -> bool {
        (self.host.managed_functions().is_type_assignable_to)(self.id, other.id).into()
    }

    pub fn is_assignable_from(&self, other: &Self) -> bool {
        (self.host.managed_functions().is_type_assignable_from)(self.id, other.id).into()
    }

    pub fn get_methods(&self) -> Vec<MethodInfo> {
        let mut method_count = 0i32;
        (self.host.managed_functions().get_type_methods)(
            self.id,
            std::ptr::null_mut(),
            &mut method_count as _,
        );

        let mut handles = Vec::with_capacity(method_count as usize);
        (self.host.managed_functions().get_type_methods)(
            self.id,
            handles.as_mut_ptr(),
            &mut method_count as _,
        );
        unsafe {
            handles.set_len(method_count as usize);
        }

        handles
            .iter()
            .map(|handle| MethodInfo::from_handle(*handle, &self.host))
            .collect()
    }

    pub fn get_fields(&self) -> Vec<FieldInfo> {
        let mut field_count = 0i32;
        (self.host.managed_functions().get_type_fields)(
            self.id,
            std::ptr::null_mut(),
            &mut field_count as _,
        );

        let mut handles = Vec::with_capacity(field_count as usize);
        (self.host.managed_functions().get_type_fields)(
            self.id,
            handles.as_mut_ptr(),
            &mut field_count as _,
        );
        unsafe {
            handles.set_len(field_count as usize);
        }

        handles
            .iter()
            .map(|handle| FieldInfo::from_handle(*handle, &self.host))
            .collect()
    }

    pub fn get_properties(&self) -> Vec<PropertyInfo> {
        let mut property_count = 0i32;
        (self.host.managed_functions().get_type_properties)(
            self.id,
            std::ptr::null_mut(),
            &mut property_count as _,
        );

        let mut handles = Vec::with_capacity(property_count as usize);
        (self.host.managed_functions().get_type_properties)(
            self.id,
            handles.as_mut_ptr(),
            &mut property_count as _,
        );
        unsafe {
            handles.set_len(property_count as usize);
        }

        handles
            .iter()
            .map(|handle| PropertyInfo::from_handle(*handle, &self.host))
            .collect()
    }

    pub fn has_attribute(&self, attribute_type: &Type) -> bool {
        (self.host.managed_functions().has_type_attribute)(self.id, attribute_type.id).into()
    }

    pub fn get_attributes(&self) -> Vec<Attribute> {
        let mut attribute_count = 0i32;
        (self.host.managed_functions().get_type_attributes)(
            self.id,
            std::ptr::null_mut(),
            &mut attribute_count as _,
        );

        let mut attribute_handles = Vec::with_capacity(attribute_count as usize);
        (self.host.managed_functions().get_type_attributes)(
            self.id,
            attribute_handles.as_mut_ptr(),
            &mut attribute_count as _,
        );
        unsafe {
            attribute_handles.set_len(attribute_count as usize);
        }

        attribute_handles
            .iter()
            .map(|handle| Attribute::from_handle(*handle, &self.host))
            .collect()
    }

    pub fn get_managed_type(&self) -> ManagedType {
        (self.host.managed_functions().get_type_managed_type)(self.id)
    }

    pub fn is_sz_array(&self) -> bool {
        (self.host.managed_functions().is_type_sz_array)(self.id).into()
    }

    pub fn get_element_type(&mut self) -> Arc<Type> {
        if self.element_type.is_none() {
            let mut element_type = Type::uninit(&self.host);
            (self.host.managed_functions().get_element_type)(self.id, &mut element_type.id as _);

            let element_type = Arc::new(element_type);
            self.host.type_cache().cache_type(element_type.clone());
            self.element_type = Some(element_type);
        }

        self.element_type.clone().unwrap()
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
        let mut object = ManagedObject::uninit(&self.host);
        object.handle = (self.host.managed_functions().create_object)(
            self.id,
            false.into(),
            std::ptr::null_mut(),
            std::ptr::null(),
            0,
        );
        // TODO: Is this the best way to do this?
        object.r#type = Some(self.host.type_cache().get_type_by_id(self.id).unwrap());

        object
    }

    fn invoke_static_method<Ret>(&self, name: &str, _args: ()) -> Ret {
        let mut method_name = CSharpNativeString::new(name);

        let result = unsafe {
            let mut result = std::mem::zeroed::<Ret>();

            (self.host.managed_functions().invoke_static_method_ret)(
                self.id,
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
                let mut object = ManagedObject::uninit(&self.host);

				let len = count_params!($($arg),+);

                let parameters = [
                    $(&mut args.$idx as *mut _ as *mut std::ffi::c_void),*
                ];

                let parameter_types = [
                    $($arg::get_managed_type()),*
                ];

                object.handle = (self.host.managed_functions().create_object)(
                    self.id,
                    false.into(),
                    &parameters as _,
                    &parameter_types as _,
                    len,
                );
                // TODO: Is this the best way to do this?
                object.r#type = Some(self.host.type_cache().get_type_by_id(self.id).unwrap());

                object
            }

			fn invoke_static_method<Ret>(&self, name: &str, mut args: ($($arg,)+)) -> Ret {
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

					(self.host.managed_functions().invoke_static_method_ret)(
						self.id,
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
