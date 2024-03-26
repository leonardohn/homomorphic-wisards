macro_rules! impl_load {
    ($ty:ident
        => $l:ident) => {
        impl_load!($ty<> => $l);
    };
    ($ty:ident
        => $l:ident ($($ai:ident : $at:ty),* $(,)?)) => {
        impl_load!($ty<> => $l ($($ai : $at),*));
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),* $(,)?>
        => $l:ident) => {
        impl_load!($ty<$($N $(: $b0 $(+$b)*)?),*> => $l ());
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),* $(,)?>
        => $l:ident ($($ai:ident : $at:ty),* $(,)?)) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> $ty<$($N),*> {
            pub fn load(
                path: impl AsRef<std::path::Path>,
                $($ai : $at),*
            ) -> std::io::Result<Self> {
                use std::os::fd::IntoRawFd;
                let file = std::fs::File::open(path)?;
                let ptr = unsafe {
                    let o_read =
                        std::ffi::CStr::from_bytes_with_nul_unchecked(b"r\0")
                            .as_ptr();
                    let c_file = libc::fdopen(file.into_raw_fd(), o_read);
                    let ptr = mosfhet_sys::$l(
                        c_file as *mut _,
                        $($ai as i32),*
                    );
                    libc::fclose(c_file);
                    ptr
                };
                Ok(Self { ptr })
            }
        }
    };
}

macro_rules! impl_load_array {
    ($ty:ident
        => $l:ident) => {
        impl_load_array!($ty<> => $l);
    };
    ($ty:ident
        => $l:ident ($($ai:ident : $at:ty),* $(,)?)) => {
        impl_load_array!($ty<> => $l ($($ai : $at),*));
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),* $(,)?>
        => $l:ident) => {
        impl_load_array!($ty<$($N $(: $b0 $(+$b)*)?),*> => $l ());
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),* $(,)?>
        => $l:ident ($($ai:ident : $at:ty),* $(,)?)) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> $ty<$($N),*> {
            pub fn load(
                path: impl AsRef<std::path::Path>,
                len: usize,
                $($ai : $at),*
            ) -> std::io::Result<Self> {
                use std::os::fd::IntoRawFd;
                let file = std::fs::File::open(path)?;
                unsafe {
                    let o_read =
                        std::ffi::CStr::from_bytes_with_nul_unchecked(b"r\0")
                            .as_ptr();
                    let c_file = libc::fdopen(file.into_raw_fd(), o_read);
                    let mut ptr = Self::new_uninit(len, $($ai),*);
                    for sample in ptr.as_slice_mut().iter_mut() {
                        mosfhet_sys::$l(
                            c_file as *mut _,
                            sample.as_ptr() as *mut _,
                        );
                    }
                    libc::fclose(c_file);
                    Ok(ptr)
                }
            }
        }
    };
}

macro_rules! impl_save {
    ($ty:ident
        => $l:ident) => {
        impl_save!($ty<> => $l);
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),*>
        => $l:ident) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> $ty<$($N),*> {
            pub fn save(
                &self,
                path: impl AsRef<std::path::Path>,
            ) -> std::io::Result<()> {
                use std::os::fd::IntoRawFd;
                let file = std::fs::File::create(path)?;
                unsafe {
                    let o_write =
                        std::ffi::CStr::from_bytes_with_nul_unchecked(b"w\0")
                            .as_ptr();
                    let c_file = libc::fdopen(file.into_raw_fd(), o_write);
                    mosfhet_sys::$l(c_file as *mut _, self.as_ptr() as *mut _);
                    libc::fclose(c_file);
                }
                Ok(())
            }
        }
    };
}

macro_rules! impl_save_array {
    ($ty:ident
        => $l:ident) => {
        impl_save_array!($ty<> => $l);
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),*>
        => $l:ident) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> $ty<$($N),*> {
            pub fn save(
                &self,
                path: impl AsRef<std::path::Path>,
            ) -> std::io::Result<()> {
                use std::os::fd::IntoRawFd;
                let file = std::fs::File::create(path)?;
                unsafe {
                    let o_write =
                        std::ffi::CStr::from_bytes_with_nul_unchecked(b"w\0")
                            .as_ptr();
                    let c_file = libc::fdopen(file.into_raw_fd(), o_write);
                    for sample in self.as_slice().iter() {
                        mosfhet_sys::$l(c_file as *mut _, sample.ptr);
                    }
                    libc::fclose(c_file);
                }
                Ok(())
            }
        }
    };
}

macro_rules! impl_drop {
    ($ty:ident
        => $l:ident) => {
        impl_drop!($ty<> => $l);
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),*>
        => $l:ident) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> Drop for $ty<$($N),*> {
            fn drop(&mut self) {
                unsafe {
                    mosfhet_sys::$l(self.as_ptr_mut() as *mut _);
                }
            }
        }
    };
}

macro_rules! impl_drop_array {
    ($ty:ident
        => $l:ident) => {
        impl_drop_array!($ty<> => $l);
    };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),*>
        => $l:ident) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> Drop for $ty<$($N),*> {
            fn drop(&mut self) {
                unsafe {
                    mosfhet_sys::$l(
                        self.as_ptr_mut() as *mut _,
                        self.len as i32,
                    );
                }
            }
        }
    };
}

macro_rules! impl_ptrs {
    ($ty:ident) => { impl_ptrs!($ty<>); };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),*>) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> $ty<$($N),*> {
            #[allow(dead_code)]
            pub(crate) fn as_ptr(&self) -> *const libc::c_void {
                self.ptr as *const _
            }

            #[allow(dead_code)]
            pub(crate) fn as_ptr_mut(&mut self) -> *mut libc::c_void {
                self.ptr as *mut _
            }
        }
    };
}

macro_rules! impl_slice_array {
    ($ty:ident) => { impl_slice_array!($ty<>); };
    ($ty:ident <$($N:ident $(: $b0:ident $(+$b:ident)*)?),*>) => {
        impl<$($N $(: $b0 $(+$b)*)?),*> $ty<$($N),*> {
            pub fn as_slice(&self) -> &[<$ty<$($N),*> as Index<usize>>::Output] {
                unsafe {
                    let ptr = self.ptr as *const _;
                    let len = self.len as usize;
                    std::slice::from_raw_parts(ptr, len)
                }
            }

            pub fn as_slice_mut(
                &mut self,
            ) -> &mut [<$ty<$($N),*> as Index<usize>>::Output] {
                unsafe {
                    std::slice::from_raw_parts_mut(
                        self.ptr as *mut _,
                        self.len as usize,
                    )
                }
            }
        }
    };
}

pub(crate) use impl_drop;
pub(crate) use impl_drop_array;
pub(crate) use impl_load;
pub(crate) use impl_load_array;
pub(crate) use impl_ptrs;
pub(crate) use impl_save;
pub(crate) use impl_save_array;
pub(crate) use impl_slice_array;
