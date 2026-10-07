use crate::alloc::Layout;
use crate::sys::pal::abi;

#[inline]
pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    unsafe { abi::MEMAllocFromDefaultHeapEx(layout.size() as u32, layout.align() as i32).cast() }
}

#[inline]
pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    let ptr = unsafe { alloc(layout) };
    if !ptr.is_null() {
        unsafe { ptr.write_bytes(0, layout.size()) }
    }
    ptr
}

#[inline]
pub unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
    unsafe { abi::MEMFreeToDefaultHeap(ptr.cast()) }
}

#[inline]
pub unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    unsafe { super::realloc_fallback(ptr, layout, new_size) }
}
