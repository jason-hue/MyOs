use core::sync::atomic::{AtomicUsize,Ordering};
use core::alloc::{GlobalAlloc,Layout};
use core::ptr::NonNull;
pub struct LinearAllocator{
    head: AtomicUsize,
    start: *mut u8,
    end: *mut u8,
}
unsafe impl Sync for LinearAllocator {}

impl LinearAllocator{
    pub const fn empty() -> LinearAllocator {
        Self{
            head: AtomicUsize::new(0),
            start: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
        }
    }
    pub fn init(&mut self,start: usize,size: usize){
        self.start = start as *mut u8;
        unsafe { self.end = self.start.add(size); }
    }
}
unsafe impl GlobalAlloc for LinearAllocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size  = layout.size();
        let mut head = self.head.load(Ordering::Relaxed);
        if head % align != 0 {
            head += align - (head % align);
        }
        let new_head = head + size;
        if self.start.add(new_head) > self.end{
            return core::ptr::null_mut();
        }
        self.head.store(new_head,Ordering::Relaxed);
        NonNull::new_unchecked(self.start.add(head) as *mut u8).as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}