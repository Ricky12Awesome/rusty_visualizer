pub type AnyError = Box<dyn std::error::Error>;
pub type AnyErrorResult<T> = std::result::Result<T, AnyError>;

#[allow(unused_macros)]
#[macro_export]
macro_rules! cstr {
    ($($arg:tt)*) => {&*std::ffi::CString::new(format!($($arg)*)).ok().map(|it| it.into_boxed_c_str()).unwrap()};
}