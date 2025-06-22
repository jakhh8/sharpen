use crate::string::CSharpNativeString;

pub type MessageCallbackFn = fn(String, MessageLevel);
pub(crate) type MessageCallbackFnInternal =
    unsafe extern "system" fn(CSharpNativeString, MessageLevel);

// TODO: Is this the correct size?
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageLevel {
    Info = 1,
    Warning = 2,
    Error = 4,
}

impl MessageLevel {
    pub fn filter(self, level: Self) -> bool {
        self <= level
    }
}

impl std::fmt::Display for MessageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageLevel::Info => f.write_str("Info"),
            MessageLevel::Warning => f.write_str("Warn"),
            MessageLevel::Error => f.write_str("Error"),
        }
    }
}
