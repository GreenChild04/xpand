use std::io::Write;
use crate::crypto::Password;

pub static mut ENABLE_LOADING_BAR: bool = true;
pub static mut VERBOSE: bool = false;

/// A simple logging system
#[derive(Debug, Clone)]
pub enum Log {
    Info(String, Option<String>),
    Warning(String, Option<String>),
    Error(String, Option<String>),
    Success(String, Option<String>),
}

impl Log {
    /// Logs a message
    #[inline]
    pub fn log(self) {
        use Log as L;
        unsafe { ENABLE_LOADING_BAR = false };
        println!("{}", match self {
            L::Info(log, details) => {
                if unsafe { !VERBOSE } { return unsafe { ENABLE_LOADING_BAR = true } };
                format!("\x1b[36;1minfo:\x1b[0m {log}{}", Self::details(6, details))
            },
            L::Warning(log, details) => format!("\x1b[33;1mwarning:\x1b[0m {log}{}", Self::details(9, details)),
            L::Error(log, details) => format!("\x1b[31;1merror:\x1b[0m {log}{}", Self::details(7, details)),
            L::Success(log, details) => format!("\x1b[32msuccess:\x1b[0m {log}{}", Self::details(9, details)),
        });
        unsafe { ENABLE_LOADING_BAR = true };
    }

    /// Priority logs a message and then crashes the program
    #[inline]
    pub fn error<T>(self) -> T {
        use Log as L;
        unsafe { ENABLE_LOADING_BAR = false };
        println!("{}", match self {
            L::Info(log, details) => format!("\x1b[36;1minfo:\x1b[0m {log}{}", Self::details(6, details)),
            L::Warning(log, details) => format!("\x1b[33;1mwarning:\x1b[0m {log}{}", Self::details(9, details)),
            L::Error(log, details) => format!("\x1b[31;1merror:\x1b[0m {log}{}", Self::details(7, details)),
            L::Success(log, details) => format!("\x1b[32msuccess:\x1b[0m {log}{}", Self::details(9, details)),
        });

        std::process::exit(1)
    }

    #[inline]
    fn details(offset: u8, details: Option<String>) -> String {
        match details {
            Some(x) => format!("\n{}\x1b[34m^ \x1b[36m{x}\x1b[0m", " ".repeat(offset as usize)),
            None => String::new(),
        }
    }
}

/// Asks the user for a password
pub fn ask_password(pass_hash: &Password) -> String {
    print!("\x1b[35;1mPassword:\x1b[0m ");
    std::io::stdout().flush().unwrap();
    
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).unwrap();
    
    if !pass_hash.verify_password(password.trim()) {
        Log::Error("incorrect password".into(), Some("while decrypting file(s)".into())).log();
        ask_password(pass_hash);
    } password.trim().to_string()
}

#[macro_export]
macro_rules! unwrap {
    (Res $details:literal: $expr:expr) => {
        match $expr {
            Ok(x) => x,
            Err(e) => $crate::log::Log::Error(e.to_string(), Some($details.to_string())).error(),
        }
    };

    (Res $expr:expr) => {
        match $expr {
            Ok(x) => x,
            Err(e) => $crate::log::Log::Error(e.to_string(), None).error(),
        }
    };

    (Opt ($error:literal) $details:literal: $expr:expr) => {
        match $expr {
            Some(x) => x,
            None => $crate::log::Log::Error($error.into(), Some($details.to_string())).error(),
        }
    };
}