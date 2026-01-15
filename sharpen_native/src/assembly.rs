use std::sync::{Arc, Mutex};

use crate::{
    TypeId,
    csharp_type::Type,
    interop_types::NativeString,
    sharpen_managed_fns::SharpenManagedFunctions,
    type_cache::{TypeCache, TypeCacheError},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssemblyLoadStatus {
    Success,
    FileNotFound,
    FileLoadFailure,
    InvalidFilePath,
    InvalidAssembly,
    UnknownError,
}

// TODO: Consider if ManagedAssembly needs to be storing this much info, and if it even really needs to exist?
pub struct ManagedAssembly {
    assembly_id: i32,
    context_id: i32,
    load_status: AssemblyLoadStatus,
    name: String,

    types: Vec<Arc<Type>>,

    managed_funcs: Arc<SharpenManagedFunctions>,
    type_cache: Arc<Mutex<TypeCache>>,
}

impl ManagedAssembly {
    pub fn new(
        context_id: i32,
        assembly_id: i32,
        load_status: AssemblyLoadStatus,
        name: String,
        types: Vec<Arc<Type>>,
        managed_funcs: Arc<SharpenManagedFunctions>,
        type_cache: Arc<Mutex<TypeCache>>,
    ) -> ManagedAssembly {
        Self {
            assembly_id,
            context_id,
            load_status,
            name,

            types,

            managed_funcs,
            type_cache,
        }
    }

    pub fn get_type(&self, class_name: &str) -> Result<Arc<Type>, TypeCacheError> {
        self.type_cache.lock().unwrap().get_type_by_name(class_name)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AssemblyLoadError {
    FileNotFound,
}

pub struct AssemblyLoadContext {
    context_id: i32,
    // TODO: Make sure this is fine compared to StableVec + C++-style referencess
    loaded_assemblies: Vec<Arc<ManagedAssembly>>,

    managed_funcs: Arc<SharpenManagedFunctions>,
    type_cache: Arc<Mutex<TypeCache>>,
}

impl AssemblyLoadContext {
    pub(crate) fn new(
        context_id: i32,
        managed_funcs: Arc<SharpenManagedFunctions>,
        type_cache: Arc<Mutex<TypeCache>>,
    ) -> Self {
        Self {
            context_id,
            loaded_assemblies: vec![],
            managed_funcs,
            type_cache,
        }
    }

    pub fn load_assembly(
        &mut self,
        path: &std::path::Path,
    ) -> Result<Arc<ManagedAssembly>, AssemblyLoadError> {
        let mut path_cs_str = NativeString::new(path.to_str().unwrap());

        let assembly_id = (self.managed_funcs.load_assembly)(self.context_id, path_cs_str.clone());
        let load_status = (self.managed_funcs.get_last_load_status)();
        let mut name = String::new();
        let mut types = Vec::new();

        // TODO: Just return Err if != Success
        if load_status == AssemblyLoadStatus::Success {
            let mut assembly_name =
                (self.managed_funcs.get_assembly_name)(self.context_id, assembly_id);
            name = assembly_name.to_string();
            NativeString::free(&mut assembly_name);

            let mut type_count = -1;
            (self.managed_funcs.get_assembly_types)(
                self.context_id,
                assembly_id,
                std::ptr::null_mut(),
                &mut type_count,
            );

            let mut type_ids = Vec::<TypeId>::with_capacity(type_count as usize);
            (self.managed_funcs.get_assembly_types)(
                self.context_id,
                assembly_id,
                type_ids.as_mut_ptr(),
                &mut type_count,
            );
            unsafe {
                type_ids.set_len(type_count as usize);
            }

            let mut type_cache = self.type_cache.lock().unwrap();
            for type_id in type_ids {
                let arc_type = Arc::new(Type::from_id(
                    type_id,
                    self.managed_funcs.clone(),
                    self.type_cache.clone(),
                ));
                types.push(arc_type.clone());
                type_cache.cache_type(arc_type);
            }
        }

        NativeString::free(&mut path_cs_str);

        let assembly = Arc::new(ManagedAssembly::new(
            self.context_id,
            assembly_id,
            load_status,
            name,
            types,
            self.managed_funcs.clone(),
            self.type_cache.clone(),
        ));
        self.loaded_assemblies.push(assembly.clone());

        Ok(assembly)
    }

    pub fn load_assembly_from_memory(
        &mut self,
        bytes: &[u8],
    ) -> Result<Arc<ManagedAssembly>, AssemblyLoadError> {
        let assembly_id = (self.managed_funcs.load_assembly_from_memory)(
            self.context_id,
            bytes.as_ptr(),
            bytes.len() as i64,
        );
        let load_status = (self.managed_funcs.get_last_load_status)();
        let mut name = String::new();
        let mut types = Vec::new();

        if load_status == AssemblyLoadStatus::Success {
            let mut assembly_name =
                (self.managed_funcs.get_assembly_name)(self.context_id, assembly_id);
            name = assembly_name.to_string();
            NativeString::free(&mut assembly_name);

            let mut type_count = -1;
            (self.managed_funcs.get_assembly_types)(
                self.context_id,
                assembly_id,
                std::ptr::null_mut(),
                &mut type_count,
            );

            let mut type_ids = Vec::<TypeId>::with_capacity(type_count as usize);
            (self.managed_funcs.get_assembly_types)(
                self.context_id,
                assembly_id,
                type_ids.as_mut_ptr(),
                &mut type_count,
            );
            unsafe {
                type_ids.set_len(type_count as usize);
            }

            let mut type_cache = self.type_cache.lock().unwrap();
            for type_id in type_ids {
                let arc_type = Arc::new(Type::from_id(
                    type_id,
                    self.managed_funcs.clone(),
                    self.type_cache.clone(),
                ));
                types.push(arc_type.clone());
                type_cache.cache_type(arc_type);
            }
        }

        let assembly = Arc::new(ManagedAssembly::new(
            self.context_id,
            assembly_id,
            load_status,
            name,
            types,
            self.managed_funcs.clone(),
            self.type_cache.clone(),
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
        (self.managed_funcs.unload_assembly_load_context)(self.context_id);
    }
}
