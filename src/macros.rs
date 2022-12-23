macro_rules! cstr {
    ($e:expr) => {
        &std::ffi::CString::new($e).unwrap()
    };
}

pub(crate) use cstr;
