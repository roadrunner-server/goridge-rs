use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::os::fd::{AsRawFd, FromRawFd};

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn pipe2_cloexec() -> io::Result<(c_int, c_int)> {
    let mut fds = mem::MaybeUninit::<[c_int; 2]>::uninit();
    let res = unsafe { libc::pipe2(fds.as_mut_ptr() as *mut c_int, libc::O_CLOEXEC) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }

    unsafe { Ok((fds.assume_init()[0], fds.assume_init()[1])) }
}

pub(crate) fn dup<F: AsRawFd>(fds: &F) -> io::Result<File> {
    let fd: i32 = fds.as_raw_fd();
    let tmp = ManuallyDrop::new(unsafe { File::from_raw_fd(fd) });

    tmp.try_clone()
}
