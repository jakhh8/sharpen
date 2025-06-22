use std::{str::FromStr, sync::Arc};

use netcorehost::pdcstring;

use crate::{
    InternalCall, TypeId, coral_managed_fns::AssemblyLoadStatus, host_instance::HostInstance,
    sharp_type::Type, string::CSharpNativeString, type_cache::TypeCacheError,
};

pub struct ManagedAssembly {
    host: HostInstance,
    assembly_id: i32,
    load_status: AssemblyLoadStatus,
    name: String,
    internal_call_name_storage: Vec<pdcstring::PdCString>,
    // TODO: Have a C# compatible InternalCall def and a proper rust one without actual pointers
    internal_calls: Vec<InternalCall>,
    types: Vec<Arc<Type>>,
}

impl ManagedAssembly {
    pub fn new(
        host: HostInstance,
        assembly_id: i32,
        load_status: AssemblyLoadStatus,
        name: String,
        types: Vec<Arc<Type>>,
    ) -> ManagedAssembly {
        Self {
            host,
            assembly_id,
            load_status,
            name,
            internal_call_name_storage: vec![],
            internal_calls: vec![],
            types,
        }
    }

    /// ## Functionality
    /// Adds the ability to call a rust function within the given class of the assembly
    ///
    /// ## Parameters
    /// `class_name`: Name of the class
    ///
    /// `variable_name`: What the function will be called in C#
    ///
    /// `fn_ptr`: Pointer to the rust function. The rust function has to be `extern "system"`, but can have any signature (except templated?).
    ///
    /// ## Safety
    /// This function will fail if the `class_name` is not found within the assembly, and will cause undefined behaviour if the `fn_ptr` is
    /// malformed, e.g. not a pointer to an `extern "system"` function. This function will also cause undefined behaviour if the rust function
    /// and the C# function declaration do not match.
    // TODO: Find a way to not use *const c_void
    pub unsafe fn add_internal_call(
        &mut self,
        class_name: &str,
        variable_name: &str,
        fn_ptr: *const unsafe extern "system" fn() -> (),
    ) {
        let assembly_qualified_name = format!("{class_name}+{variable_name}, {}", self.name);
        let name = pdcstring::PdCString::from_str(&assembly_qualified_name).unwrap();

        self.internal_calls.push(InternalCall {
            name: name.as_ptr(),
            native_function_ptr: fn_ptr as _,
        });
        self.internal_call_name_storage.push(name);
    }

    // TODO: Get Result<(), Err>
    pub fn upload_internal_calls(&self) {
        (self.host.managed_functions().set_internal_calls)(
            self.internal_calls.as_ptr() as *mut _,
            self.internal_calls.len() as i32,
        );
    }

    pub fn get_type(&self, class_name: &str) -> Result<Arc<Type>, TypeCacheError> {
        self.host.type_cache().get_type_by_name(class_name)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AssemblyLoadError {
    FileNotFound,
}

pub struct AssemblyLoadContext {
    context_id: i32,
    host: HostInstance,
    // TODO: Make sure this is fine compared to StableVec + C++-style referencess
    loaded_assemblies: Vec<Arc<ManagedAssembly>>,
}

impl AssemblyLoadContext {
    pub fn new(context_id: i32, host: &HostInstance) -> Self {
        Self {
            context_id,
            host: host.clone(),
            loaded_assemblies: vec![],
        }
    }

    pub fn load_assembly(
        &mut self,
        path: &std::path::Path,
    ) -> Result<Arc<ManagedAssembly>, AssemblyLoadError> {
        let mut path_cs_str = CSharpNativeString::new(path.to_str().unwrap());

        let managed_functions = self.host.managed_functions();

        let assembly_id = (managed_functions.load_assembly)(self.context_id, path_cs_str.clone());
        let load_status = (managed_functions.get_last_load_status)();
        let mut name = String::new();
        let mut types = Vec::new();

        // TODO: Just return Err if != Success
        if load_status == AssemblyLoadStatus::Success {
            let mut assembly_name = (managed_functions.get_assembly_name)(assembly_id);
            name = assembly_name.to_string();
            CSharpNativeString::free(&mut assembly_name);

            let mut type_count = -1;
            (managed_functions.get_assembly_types)(
                assembly_id,
                std::ptr::null_mut(),
                &mut type_count,
            );

            let mut type_ids = Vec::<TypeId>::with_capacity(type_count as usize);
            (managed_functions.get_assembly_types)(
                assembly_id,
                type_ids.as_mut_ptr(),
                &mut type_count,
            );
            unsafe {
                type_ids.set_len(type_count as usize);
            }

            for type_id in type_ids {
                let arc_type = Arc::new(Type::from_id(type_id, &self.host));
                types.push(arc_type.clone());
                self.host.type_cache().cache_type(arc_type);
            }
        }

        CSharpNativeString::free(&mut path_cs_str);

        let assembly = Arc::new(ManagedAssembly::new(
            self.host.clone(),
            assembly_id,
            load_status,
            name,
            types,
        ));
        self.loaded_assemblies.push(assembly.clone());

        Ok(assembly)
    }

    pub fn load_assembly_from_memory(
        &mut self,
        bytes: &[u8],
    ) -> Result<Arc<ManagedAssembly>, AssemblyLoadError> {
        let managed_functions = self.host.managed_functions();

        let assembly_id = (managed_functions.load_assembly_from_memory)(
            self.context_id,
            bytes.as_ptr(),
            bytes.len() as i64,
        );
        let load_status = (managed_functions.get_last_load_status)();
        let mut name = String::new();
        let mut types = Vec::new();

        if load_status == AssemblyLoadStatus::Success {
            let mut assembly_name = (managed_functions.get_assembly_name)(assembly_id);
            name = assembly_name.to_string();
            CSharpNativeString::free(&mut assembly_name);

            let mut type_count = 0;
            (managed_functions.get_assembly_types)(
                assembly_id,
                std::ptr::null_mut(),
                &mut type_count,
            );

            let mut type_ids = Vec::<TypeId>::with_capacity(type_count as usize);
            (managed_functions.get_assembly_types)(
                assembly_id,
                type_ids.as_mut_ptr(),
                &mut type_count,
            );

            for type_id in type_ids {
                let arc_type = Arc::new(Type::from_id(type_id, &self.host));
                types.push(arc_type.clone());
                self.host.type_cache().cache_type(arc_type);
            }
        }

        let assembly = Arc::new(ManagedAssembly::new(
            self.host.clone(),
            assembly_id,
            load_status,
            name,
            types,
        ));
        self.loaded_assemblies.push(assembly.clone());

        Ok(assembly)
    }

    pub fn context_id(&self) -> i32 {
        self.context_id
    }

    pub fn loaded_assemblies(&self) -> &Vec<Arc<ManagedAssembly>> {
        &self.loaded_assemblies
    }
}

impl Drop for AssemblyLoadContext {
    fn drop(&mut self) {
        self.host.unload_assembly_load_context(self);
    }
}
