use crate::{Bool32, string::CSharpNativeString};

pub trait FromCSharp<T> {
    fn from_csharp(csharp_value: T) -> Self;
}

impl FromCSharp<Bool32> for bool {
    fn from_csharp(csharp_value: Bool32) -> Self {
        // TODO: Figure out the specifics of Bool32 (is true just Bool32 > 0)
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
impl FromCSharp<CSharpNativeString> for String {
    fn from_csharp(csharp_value: CSharpNativeString) -> Self {
        csharp_value.to_string()
    }
}

impl Into<String> for CSharpNativeString {
    fn into(self) -> String {
        self.to_string()
    }
}

// TODO: This will implement FromCSharp<String> for String which is unwanted(?)
impl<T> FromCSharp<T> for T {
    fn from_csharp(csharp_value: T) -> Self {
        csharp_value
    }
}
