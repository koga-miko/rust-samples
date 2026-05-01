mod raw;

use std::error;
use std::fmt;
use std::result;
use std::os::raw::c_int;
use std::ffi::CStr;
use std::path::Path;
use std::ptr;

#[derive(Debug)]
pub struct Error {
    code: i32,
    message: String,
    class: i32
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        self.message.fmt(f)
    }
}

impl error::Error for Error {  }

pub type Result<T> = result::Result<T, Error>;

fn check(code: c_int) -> Result<c_int> {
    if code >= 0 {
        return Ok(code);
    }

    unsafe {
        let error = raw::giterr_last();
        let message = CStr::from_ptr((*error).message).to_string_lossy().into_owned();
        Err(Error {
            code: code as i32,
            message,
            class: (*error).klass as i32
        })
    }
}

pub struct Repository {
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();

        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();
        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }
        Ok(Repository { raw: repo })
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}

fn ensure_initialized() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        unsafe {
            check(raw::git_libgit2_init()).expect("initializing libgit2 failed");
            assert_eq!(libc::atexit(shutdown), 0);
        }
    })
}

extern "C" fn shutdown() {
    unsafe {
        if let Err(e) = check(raw::git_libgit2_shutdown()) {
            eprintln!("libgit2 shutdown failed: {}", e);
            std::process::abort();
        }
    }
}

use std::ffi::CString;

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF=8", path.display());
            Err(message.into())
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Error {
        Error { code: -1, message, class: 0 }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Error {
        Error { code: -1, message: e.to_string(), class: 0 }
    }
}

pub struct Oid {
    pub raw: raw::git_oid
}

use std::mem;
use std::os::raw::c_char;

impl Repository {
    pub fn reference_name_to_id(&self, name: &str) -> Result<Oid> {
        let name = CString::new(name)?;
        unsafe {
            let oid = {
                let mut oid = mem::MaybeUninit::uninit();
                check(raw::git_reference_name_to_id(
                    oid.as_mut_ptr(),
                    self.raw,
                    name.as_ptr() as *const c_char
                ))?;
                oid.assume_init()
            };
            Ok(Oid { raw: oid })
        }
    }
}

use std::marker::PhantomData;

pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

impl Repository {
    pub fn find_commit<'repo>(&'repo self, oid: &Oid) -> Result<Commit<'repo>> {
        let mut commit = ptr::null_mut();
        unsafe {
            check(raw::git_commit_lookup(&mut commit, self.raw, &oid.raw))?;
        }
        Ok(Commit { raw: commit, _marker: PhantomData })
    }
}

impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            raw::git_commit_free(self.raw);
        }
    }
}

impl<'repo> Commit<'repo> {
    pub fn author(&self) -> Signature {
        unsafe {
            Signature {
                raw: raw::git_commit_author(self.raw),
                _maker: PhantomData
            }
        }
    }
    
    pub fn message(&self) -> Option<&str> {
        unsafe {
            let message = raw::git_commit_message(self.raw);
            char_ptr_to_str(self, message)
        }
    }
}

pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _maker: PhantomData<&'text str>
}

impl<'text> Signature<'text> {
    pub fn name(&self) -> Option<&str> {
        unsafe {
            char_ptr_to_str(self, (*self.raw).name)
        }
    }

    pub fn email(&self) -> Option<&str> {
        unsafe {
            char_ptr_to_str(self, (*self.raw).email)
        }
    }
}

unsafe fn char_ptr_to_str<'o, T: 'o>(_owner: &'o T, ptr: *const c_char) -> Option<&'o str> {
    if ptr.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(ptr).to_str().ok()
        }
    }
}