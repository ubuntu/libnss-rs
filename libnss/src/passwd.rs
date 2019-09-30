use crate::interop::CBuffer;

pub struct Passwd {
    pub name: String,
    pub passwd: String,
    pub uid: libc::uid_t,
    pub gid: libc::gid_t,
    pub gecos: String,
    pub dir: String,
    pub shell: String,
}

impl Passwd {
    pub unsafe fn to_c_passwd(
        self,
        pwbuf: *mut CPasswd,
        buffer: &mut CBuffer,
    ) -> std::io::Result<()> {
        (*pwbuf).name = buffer.write_str(self.name)?;
        (*pwbuf).passwd = buffer.write_str(self.passwd)?;
        (*pwbuf).uid = self.uid;
        (*pwbuf).gid = self.gid;
        (*pwbuf).gecos = buffer.write_str(self.gecos)?;
        (*pwbuf).dir = buffer.write_str(self.dir)?;
        (*pwbuf).shell = buffer.write_str(self.shell)?;
        Ok(())
    }
}

pub trait PasswdHooks {
    fn get_all_entries() -> Vec<Passwd>;

    fn get_entry_by_uid(uid: libc::uid_t) -> Option<Passwd>;

    fn get_entry_by_name(name: String) -> Option<Passwd>;
}

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct CPasswd {
    pub name: *mut libc::c_char,
    pub passwd: *mut libc::c_char,
    pub uid: libc::uid_t,
    pub gid: libc::gid_t,
    pub gecos: *mut libc::c_char,
    pub dir: *mut libc::c_char,
    pub shell: *mut libc::c_char,
}

#[macro_export]
macro_rules! libnss_passwd_hooks {
($mod_ident:ident, $hooks_ident:ident) => (
    paste::item! {
        pub use self::[<libnss_passwd_ $mod_ident _hooks_impl>]::*;
        mod [<libnss_passwd_ $mod_ident _hooks_impl>] {
            #![allow(non_upper_case_globals)]

            use std::ffi::CStr;
            use std::str;
            use std::sync::{Mutex, MutexGuard};
            use $crate::interop::{CBuffer, Iterator, NssStatus};
            use $crate::passwd::{CPasswd, Passwd, PasswdHooks};

            lazy_static! {
            static ref [<PASSWD_ $mod_ident _ITERATOR>]: Mutex<Iterator<Passwd>> = Mutex::new(Iterator::<Passwd>::new());
            }

            #[no_mangle]
            extern "C" fn [<_nss_ $mod_ident _setpwent>]() -> libc::c_int {
                let mut iter: MutexGuard<Iterator<Passwd>> = [<PASSWD_ $mod_ident _ITERATOR>].lock().unwrap();
                iter.open(super::$hooks_ident::get_all_entries());
                NssStatus::Success.to_c()
            }

            #[no_mangle]
            extern "C" fn [<_nss_ $mod_ident _endpwent>]() -> libc::c_int {
                let mut iter: MutexGuard<Iterator<Passwd>> = [<PASSWD_ $mod_ident _ITERATOR>].lock().unwrap();
                iter.close();

                NssStatus::Success.to_c()
            }

            #[no_mangle]
            unsafe extern "C" fn [<_nss_ $mod_ident _getpwent_r>](pwbuf: *mut CPasswd, buf: *mut libc::c_char, buflen: libc::size_t,
                                                                  errnop: *mut libc::c_int) -> libc::c_int {
                let mut iter: MutexGuard<Iterator<Passwd>> = [<PASSWD_ $mod_ident _ITERATOR>].lock().unwrap();
                match iter.next() {
                    None => $crate::interop::NssStatus::NotFound.to_c(),
                    Some(entry) => {
                        let mut buffer = CBuffer::new(buf as *mut libc::c_void, buflen);
                        buffer.clear();

                        match entry.to_c_passwd(pwbuf, &mut buffer) {
                            Err(e) => {
                                match e.raw_os_error() {
                                   Some(e) =>{
                                       *errnop = e;
                                       NssStatus::TryAgain.to_c()
                                   },
                                   None => {
                                       *errnop = libc::ENOENT;
                                       NssStatus::Unavail.to_c()
                                   }
                               }
                            },
                            Ok(_) => {
                                *errnop = 0;
                                NssStatus::Success.to_c()
                            }
                        }
                    }
                }
            }

            #[no_mangle]
            unsafe extern "C" fn [<_nss_ $mod_ident _getpwuid_r>](uid: libc::uid_t, pwbuf: *mut CPasswd, buf: *mut libc::c_char,
                                                           buflen: libc::size_t, errnop: *mut libc::c_int) -> libc::c_int {
                match super::$hooks_ident::get_entry_by_uid(uid) {
                    Some(val) => {
                        let mut buffer = CBuffer::new(buf as *mut libc::c_void, buflen);
                        buffer.clear();

                        match val.to_c_passwd(pwbuf, &mut buffer) {
                            Err(e) => {
                                match e.raw_os_error() {
                                   Some(e) =>{
                                       *errnop = e;
                                       NssStatus::TryAgain.to_c()
                                   },
                                   None => {
                                       *errnop = libc::ENOENT;
                                       NssStatus::Unavail.to_c()
                                   }
                               }
                            },
                            Ok(_) => {
                                *errnop = 0;
                                NssStatus::Success.to_c()
                            }
                        }
                    },
                    None => NssStatus::NotFound.to_c()
                }
            }

            #[no_mangle]
            unsafe extern "C" fn [<_nss_ $mod_ident _getpwnam_r>](name_: *const libc::c_char, pwbuf: *mut CPasswd, buf: *mut libc::c_char,
                                                           buflen: libc::size_t, errnop: *mut libc::c_int) -> libc::c_int {
                let cstr = CStr::from_ptr(name_);

                match str::from_utf8(cstr.to_bytes()) {
                    Ok(name) => match super::$hooks_ident::get_entry_by_name(name.to_string()) {
                        Some(val) => {
                            let mut buffer = CBuffer::new(buf as *mut libc::c_void, buflen);
                            buffer.clear();

                            match val.to_c_passwd(pwbuf, &mut buffer) {
                                Err(e) => {
                                    match e.raw_os_error() {
                                       Some(e) =>{
                                           *errnop = e;
                                           NssStatus::TryAgain.to_c()
                                       },
                                       None => {
                                           *errnop = libc::ENOENT;
                                           NssStatus::Unavail.to_c()
                                       }
                                   }
                                },
                                Ok(_) => {
                                    *errnop = 0;
                                    NssStatus::Success.to_c()
                                }
                            }
                        },
                        None => NssStatus::NotFound.to_c()
                    },
                    Err(_) => NssStatus::NotFound.to_c()
                }
            }
        }
    }
)
}