use std::sync::Arc;

use crate::{
    ManagedHandle, TypeAccessibility, from_csharp::FromCSharp, host_instance::HostInstance,
    sharp_type::Type, string::CSharpNativeString,
};

pub struct Attribute {
    handle: ManagedHandle,
    r#type: Option<Arc<Type>>,

    host: HostInstance,
}

impl Attribute {
    pub fn from_handle(handle: ManagedHandle, host: &HostInstance) -> Self {
        Self {
            handle,
            r#type: None,
            host: host.clone(),
        }
    }

    pub fn get_type(&mut self) -> Arc<Type> {
        if self.r#type.is_none() {
            let mut r#type = Type::uninit(&self.host);
            (self.host.managed_functions().get_attribute_type)(self.handle, &mut r#type.id as _);

            let r#type = Arc::new(r#type);
            self.host.type_cache().cache_type(r#type.clone());
            self.r#type = Some(r#type);
        }

        self.r#type.clone().unwrap()
    }

    pub fn get_field_value<CSharpType, Ret: FromCSharp<CSharpType>>(&self, name: &str) -> Ret {
        let mut result: CSharpType = unsafe { std::mem::zeroed() };

        let mut field_name = CSharpNativeString::new(name);
        (self.host.managed_functions().get_attribute_field_value)(
            self.handle,
            field_name.clone(),
            &mut result as *mut CSharpType as _,
        );
        CSharpNativeString::free(&mut field_name);

        Ret::from_csharp(result)
    }
}

pub struct MethodInfo {
    handle: ManagedHandle,
    return_type: Option<Arc<Type>>,
    parameter_types: Vec<Arc<Type>>,

    host: HostInstance,
}

impl MethodInfo {
    pub fn from_handle(handle: ManagedHandle, host: &HostInstance) -> Self {
        Self {
            handle,
            return_type: None,
            parameter_types: Vec::new(),
            host: host.clone(),
        }
    }

    pub fn get_name(&self) -> CSharpNativeString {
        (self.host.managed_functions().get_method_info_name)(self.handle)
    }

    pub fn get_return_type(&mut self) -> Arc<Type> {
        if self.return_type.is_none() {
            let mut return_type = Type::uninit(&self.host);
            (self.host.managed_functions().get_method_info_return_type)(
                self.handle,
                &mut return_type.id,
            );

            let return_type = Arc::new(return_type);
            self.host.type_cache().cache_type(return_type.clone());
            self.return_type = Some(return_type);
        }

        self.return_type.clone().unwrap()
    }

    pub fn get_parameter_types(&mut self) -> &Vec<Arc<Type>> {
        if self.parameter_types.is_empty() {
            let mut parameter_count = 0i32;
            (self
                .host
                .managed_functions()
                .get_method_info_parameter_types)(
                self.handle,
                std::ptr::null_mut(),
                &mut parameter_count as _,
            );

            let mut parameter_type_ids = Vec::with_capacity(parameter_count as usize);
            (self
                .host
                .managed_functions()
                .get_method_info_parameter_types)(
                self.handle,
                parameter_type_ids.as_mut_ptr(),
                &mut parameter_count as _,
            );
            unsafe {
                parameter_type_ids.set_len(parameter_count as usize);
            }

            self.parameter_types = parameter_type_ids
                .iter()
                .map(|id| {
                    let r#type = Arc::new(Type::from_id(*id, &self.host));
                    self.host.type_cache().cache_type(r#type.clone());
                    r#type
                })
                .collect();
        }

        &self.parameter_types
    }

    pub fn get_accessibility(&self) -> TypeAccessibility {
        (self.host.managed_functions().get_method_info_accessibility)(self.handle)
    }

    pub fn get_attributes(&self) -> Vec<Attribute> {
        let mut attribute_count = 0i32;
        (self.host.managed_functions().get_method_info_attributes)(
            self.handle,
            std::ptr::null_mut(),
            &mut attribute_count as _,
        );

        let mut attribute_handles = Vec::with_capacity(attribute_count as usize);
        (self.host.managed_functions().get_method_info_attributes)(
            self.handle,
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
}

pub struct FieldInfo {
    handle: ManagedHandle,
    r#type: Option<Arc<Type>>,

    host: HostInstance,
}

impl FieldInfo {
    pub fn from_handle(handle: ManagedHandle, host: &HostInstance) -> Self {
        Self {
            handle,
            r#type: None,
            host: host.clone(),
        }
    }

    pub fn get_name(&self) -> CSharpNativeString {
        (self.host.managed_functions().get_field_info_name)(self.handle)
    }

    pub fn get_type(&mut self) -> Arc<Type> {
        if self.r#type.is_none() {
            let mut r#type = Type::uninit(&self.host);
            (self.host.managed_functions().get_field_info_type)(self.handle, &mut r#type.id);

            let r#type = Arc::new(r#type);
            self.host.type_cache().cache_type(r#type.clone());
            self.r#type = Some(r#type);
        }

        self.r#type.clone().unwrap()
    }

    pub fn get_accessibility(&self) -> TypeAccessibility {
        (self.host.managed_functions().get_field_info_accessibility)(self.handle)
    }

    pub fn get_attributes(&self) -> Vec<Attribute> {
        let mut attribute_count = 0i32;
        (self.host.managed_functions().get_field_info_attributes)(
            self.handle,
            std::ptr::null_mut(),
            &mut attribute_count as _,
        );

        let mut attribute_handles = Vec::with_capacity(attribute_count as usize);
        (self.host.managed_functions().get_field_info_attributes)(
            self.handle,
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
}

pub struct PropertyInfo {
    handle: ManagedHandle,
    r#type: Option<Arc<Type>>,

    host: HostInstance,
}

impl PropertyInfo {
    pub fn from_handle(handle: ManagedHandle, host: &HostInstance) -> Self {
        Self {
            handle,
            r#type: None,
            host: host.clone(),
        }
    }

    pub fn get_name(&self) -> CSharpNativeString {
        (self.host.managed_functions().get_property_info_name)(self.handle)
    }

    pub fn get_type(&mut self) -> Arc<Type> {
        if self.r#type.is_none() {
            let mut r#type = Type::uninit(&self.host);
            (self.host.managed_functions().get_property_info_type)(self.handle, &mut r#type.id);

            let r#type = Arc::new(r#type);
            self.host.type_cache().cache_type(r#type.clone());
            self.r#type = Some(r#type);
        }

        self.r#type.clone().unwrap()
    }

    pub fn get_attributes(&self) -> Vec<Attribute> {
        let mut attribute_count = 0i32;
        (self.host.managed_functions().get_property_info_attributes)(
            self.handle,
            std::ptr::null_mut(),
            &mut attribute_count as _,
        );

        let mut attribute_handles = Vec::with_capacity(attribute_count as usize);
        (self.host.managed_functions().get_property_info_attributes)(
            self.handle,
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
}
