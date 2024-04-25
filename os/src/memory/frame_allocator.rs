use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Debug, Formatter};
use lazy_static::lazy_static;
use crate::memory::address::PhysAddr;
use crate::config::MEMORY_END;
use crate::memory::address::PhysPageNum;
use crate::sync::UPsafeCell;

trait FrameAllocator{
    fn new()->Self;
    fn alloc(&mut self)->Option<PhysPageNum>;
    fn dealloc(&mut self,ppn: PhysPageNum);
}
pub struct StackFrameAllocator{
    current_ppn: usize,//空闲内存的起始物理页号
    end_ppn: usize,//空闲内存的结束物理页号
    recycled: Vec<usize>,
}
impl FrameAllocator for StackFrameAllocator{
    fn new() -> Self {
        Self{
            current_ppn: 0,
            end_ppn: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current_ppn == self.end_ppn {
            None
        } else {
            self.current_ppn += 1;
            Some((self.current_ppn - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current_ppn || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}
impl StackFrameAllocator{
    pub fn init(&mut self,left: PhysPageNum,right:PhysPageNum){
        self.current_ppn = left.0;
        self.end_ppn = right.0;
    }
}
type FrameAllocatorImpl = StackFrameAllocator;
lazy_static!{
    pub static ref FRAME_ALLOCATOR: UPsafeCell<FrameAllocatorImpl> = unsafe{
        UPsafeCell::new(FrameAllocatorImpl::new())
    };
}
pub fn init_frame_allocator(){
    extern "C"{
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}
pub fn frame_alloc()->Option<FrameTracker>{
    FRAME_ALLOCATOR.exclusive_access().alloc().map(|ppn| FrameTracker::new(ppn))
}
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .dealloc(ppn);
}
#[derive(Clone)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}
impl FrameTracker{
    pub fn new(ppn: PhysPageNum) -> FrameTracker {
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self{ppn}
    }
}
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}
#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
