use super::vmem;

pub fn kmalloc(size: usize) -> Option<*mut u8> {
    vmem::vmalloc(size)
}

pub fn kfree(ptr: *mut u8) -> bool {
    vmem::vfree(ptr)
}

pub fn ksize(ptr: *const u8) -> Option<usize> {
    vmem::vsize(ptr)
}
