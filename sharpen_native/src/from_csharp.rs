use crate::{Bool32, interop_types::NativeString};

pub trait FromCSharp<T> {
    fn from_csharp(csharp_value: T) -> Self;
}

impl FromCSharp<Bool32> for bool {
    fn from_csharp(csharp_value: Bool32) -> Self {
        csharp_value.0 > 0
    }
}

impl Into<bool> for Bool32 {
    fn into(self) -> bool {
        bool::from_csharp(self)
    }
}

impl Into<Bool32> for bool {
    fn into(self) -> Bool32 {
        Bool32(self as std::ffi::c_ulong)
    }
}

// TODO: Consider cleanup?
impl FromCSharp<NativeString> for String {
    fn from_csharp(csharp_value: NativeString) -> Self {
        csharp_value.to_string()
    }
}

impl Into<String> for NativeString {
    fn into(self) -> String {
        self.to_string()
    }
}

// This makes a 'passthrough' implementation of FromCSharp for types which do not need translation
macro_rules! impl_from_csharp {
    ($type:ty) => {
        impl FromCSharp<$type> for $type {
            fn from_csharp(csharp_value: $type) -> Self {
                csharp_value
            }
        }
    };
}

impl_from_csharp!(i8);
impl_from_csharp!(i16);
impl_from_csharp!(i32);
impl_from_csharp!(i64);
impl_from_csharp!(u8);
impl_from_csharp!(u16);
impl_from_csharp!(u32);
impl_from_csharp!(u64);
