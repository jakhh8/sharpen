use crate::string::CSharpNativeString;

pub type MessageCallbackFn = fn(String, MessageLevel);
pub(crate) type MessageCallbackFnInternal = fn(CSharpNativeString, MessageLevel);

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

pub struct MessageLevelFilter {
    filter: u8,
}

/* impl MessageLevelFilter {
    const INFO: Self = Self { filter: 1 << 0 };
    const WARNING: Self = Self { filter: 1 << 1 };
    const ERROR: Self = Self { filter: 1 << 2 };
    const ALL: Self = Self::INFO | Self::WARNING | Self::ERROR;

    pub fn new() -> Self {
        Self { filter: 0 }
    }

    pub fn allow(self, level: u8) -> Self {
        Self {
            filter: self.filter | level,
        }
    }
}

impl BitAnd for MessageLevelFilter {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            filter: self.filter & rhs.filter,
        }
    }
}

impl BitOr for MessageLevelFilter {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            filter: self.filter | rhs.filter,
        }
    }
} */
