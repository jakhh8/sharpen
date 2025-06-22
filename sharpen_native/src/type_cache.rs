use std::{collections::HashMap, sync::Arc};

use crate::{TypeId, sharp_type::Type};

#[derive(Debug, Clone, Copy)]
pub enum TypeCacheError {
    TypeNotFound,
}

pub struct TypeCache {
    name_cache: HashMap<String, Arc<Type>>,
    id_cache: HashMap<TypeId, Arc<Type>>,
}

impl TypeCache {
    pub fn new() -> Self {
        Self {
            name_cache: HashMap::new(),
            id_cache: HashMap::new(),
        }
    }

    pub fn cache_type(&mut self, r#type: Arc<Type>) {
        self.name_cache
            .insert(r#type.get_full_name(), r#type.clone());
        self.id_cache.insert(r#type.get_type_id(), r#type.clone());
    }

    pub fn get_type_by_name(&self, name: &str) -> Result<Arc<Type>, TypeCacheError> {
        self.name_cache
            .get(name)
            .map(|r#type| r#type.clone())
            .ok_or(TypeCacheError::TypeNotFound)
    }

    pub fn get_type_by_id(&self, id: TypeId) -> Result<Arc<Type>, TypeCacheError> {
        self.id_cache
            .get(&id)
            .map(|r#type| r#type.clone())
            .ok_or(TypeCacheError::TypeNotFound)
    }

    pub fn clear(&mut self) {
        self.name_cache.clear();
        self.id_cache.clear();
    }
}
