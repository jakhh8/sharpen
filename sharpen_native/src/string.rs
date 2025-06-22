use netcorehost::pdcstring::{self, PdChar, windows::widestring};

use crate::Bool32;

#[repr(C)]
#[derive(Clone)]
pub struct CSharpNativeString {
    // TODO: Remove pub(crate)
    pub(crate) string: *const PdChar,
    /// NOTE: Unused, purely for C#
    // TODO: Remove pub(crate)
    pub(crate) _is_disposed: Bool32,
}

impl CSharpNativeString {
    pub fn new(str: &str) -> Self {
        let mut res = Self {
            string: std::ptr::null(),
            _is_disposed: 0,
        };

        res.assign(str);

        res
    }

    pub fn assign(&mut self, str: &str) {
        if !self.string.is_null() {
            Self::dealloc(self.string);
        }

        self.string = Self::alloc(widestring::WideCString::from_str_truncate(str));
    }

    pub fn free(string: &mut Self) {
        if string.string.is_null() {
            return;
        }

        Self::dealloc(string.string);
        string.string = std::ptr::null();
    }

    pub fn to_string(&self) -> String {
        unsafe { widestring::WideCString::from_ptr_str(self.string).to_string_lossy() }
    }
}

impl CSharpNativeString {
    fn alloc(string: widestring::WideCString) -> *const PdChar {
        unsafe {
            let ptr = std::alloc::alloc_zeroed(
                std::alloc::Layout::array::<PdChar>(string.len() + 1)
                    .expect("Could not get Layout for CSharpNativeString!"),
            ) as *mut PdChar;

            std::ptr::copy_nonoverlapping(string.as_ptr(), ptr, string.len() + 1);

            ptr
        }
    }

    fn dealloc(string: *const PdChar) {
        unsafe {
            let mut len = 0;
            let mut ptr = string;
            while ptr.read() != 0 {
                len += 1;
                ptr = ptr.byte_add(1);
            }
            len += 1; // Nul-terminator

            std::alloc::dealloc(
                string as *mut _,
                std::alloc::Layout::array::<PdChar>(len)
                    .expect("Could not get Layout for CSharpNativeString!"),
            );
        }
    }
}

impl PartialEq for CSharpNativeString {
    fn eq(&self, other: &Self) -> bool {
        if self.string == other.string {
            return true;
        }

        if self.string.is_null() || other.string.is_null() {
            return false;
        }

        unsafe {
            pdcstring::PdCString::from_str_ptr(self.string)
                == pdcstring::PdCString::from_str_ptr(other.string)
        }
    }
}
impl Eq for CSharpNativeString {}

#[repr(C)]
pub struct ScopedCSharpNativeString {
    string: CSharpNativeString,
}

impl ScopedCSharpNativeString {
    pub fn new(string: CSharpNativeString) -> Self {
        Self { string }
    }

    pub fn from_str(str: &str) -> Self {
        Self {
            string: CSharpNativeString::new(str),
        }
    }

    pub fn inner(&self) -> CSharpNativeString {
        self.string.clone()
    }
}

impl Drop for ScopedCSharpNativeString {
    fn drop(&mut self) {
        CSharpNativeString::free(&mut self.string);
    }
}

impl PartialEq for ScopedCSharpNativeString {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}
impl Eq for ScopedCSharpNativeString {}
