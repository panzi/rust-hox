// This file is part of rust-hox.
//
// rust-hox is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-hox is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-hox.  If not, see <https://www.gnu.org/licenses/>.

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

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn mem(&self) -> &[u8] {
        unsafe {
            std::ptr::slice_from_raw_parts::<u8>(self.ptr.cast(), self.size).as_ref().unwrap()
        }
    }

    #[allow(dead_code)]
    pub fn close(self) -> std::io::Result<()> {
        let result = unsafe {
            libc::munmap(self.ptr, self.size as libc::size_t)
        };

        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}

impl<'a> AsRef<[u8]> for MMap<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.mem()
    }
}

impl Drop for MMap<'_> {
    fn drop(&mut self) {
        let result = unsafe {
            libc::munmap(self.ptr, self.size as libc::size_t) 
        };
        if result != 0 {
            panic!("munmap(): {}", std::io::Error::last_os_error());
        }
    }
}
