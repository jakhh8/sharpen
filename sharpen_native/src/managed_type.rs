// TODO: Is this the correct size?
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManagedType {
    Unknown,

    SByte,
    Byte,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,

    Float,
    Double,

    Bool,

    String,

    Pointer,
}

pub trait GetManagedType {
    fn get_managed_type() -> ManagedType;
}

fn is_type_a_ptr<T>() -> bool {
    let name = std::any::type_name::<T>();

    if name.starts_with("*mut") || name.starts_with("*const") {
        true
    } else {
        false
    }
}

impl<T: 'static> GetManagedType for T {
    fn get_managed_type() -> ManagedType {
        if is_type_a_ptr::<T>() {
            return ManagedType::Pointer;
        }

        match std::any::TypeId::of::<T>() {
            id if id == std::any::TypeId::of::<i8>() => ManagedType::SByte,
            id if id == std::any::TypeId::of::<u8>() => ManagedType::Byte,
            id if id == std::any::TypeId::of::<i16>() => ManagedType::Short,
            id if id == std::any::TypeId::of::<u16>() => ManagedType::UShort,
            id if id == std::any::TypeId::of::<i32>() => ManagedType::Int,
            id if id == std::any::TypeId::of::<u32>() => ManagedType::UInt,
            id if id == std::any::TypeId::of::<i64>() => ManagedType::Long,
            id if id == std::any::TypeId::of::<u64>() => ManagedType::ULong,
            id if id == std::any::TypeId::of::<f32>() => ManagedType::Float,
            id if id == std::any::TypeId::of::<f64>() => ManagedType::Double,
            id if id == std::any::TypeId::of::<bool>() => ManagedType::Bool,
            id if id == std::any::TypeId::of::<String>() => ManagedType::String,
            _ => ManagedType::Unknown,
        }
    }
}
