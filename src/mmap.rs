use std::os::unix::io::AsRawFd;

pub struct MMap<'a> {
    ptr: *mut libc::c_void,
    size: usize,
    phantom: std::marker::PhantomData<&'a libc::c_void>,
}

impl<'a> MMap<'a> {
    pub fn new(file: &'a mut std::fs::File, offset: u64, size: usize) -> std::io::Result<Self> {
        if size > libc::size_t::MAX as usize || offset > libc::off_t::MAX as u64 {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }

        let fd = file.as_raw_fd();

        unsafe {
            let ptr = libc::mmap(std::ptr::null_mut(), size as libc::size_t, libc::PROT_READ, libc::MAP_PRIVATE, fd, offset as libc::off_t);

            if ptr == libc::MAP_FAILED {
                return Err(std::io::Error::last_os_error());
            }

            Ok(Self {
                ptr,
                size,
                phantom: std::marker::PhantomData,
            })
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn mem(&self) -> &[u8] {
        unsafe {
            std::ptr::slice_from_raw_parts::<u8>(self.ptr.cast(), self.size).as_ref().unwrap()
        }
    }
}

impl Drop for MMap<'_> {
    fn drop(&mut self) {
        let result = unsafe {
            libc::munmap(self.ptr, self.size as libc::size_t) 
        };
        if result != 0 {
            panic!(std::io::Error::last_os_error());
        }
    }
}
