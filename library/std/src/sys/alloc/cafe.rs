use cafe_sys::coreinit::mem::default_heap;

use crate::alloc::Layout;
use crate::ptr;

#[inline]
pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    let Some(alloc) = (unsafe { default_heap::ALLOC_FROM_DEFAULT_HEAP_EX }) else {
        return ptr::null_mut();
    };
    unsafe { alloc(layout.size() as u32, layout.align().max(4) as i32).cast() }
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
    if let Some(free) = unsafe { default_heap::FREE_TO_DEFAULT_HEAP } {
        unsafe { free(ptr.cast()) }
    }
}

#[inline]
pub unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    unsafe { super::realloc_fallback(ptr, layout, new_size) }
}
