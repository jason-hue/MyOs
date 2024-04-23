use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::memory::memory_set::{KERNEL_SPACE, MapPermission};
use crate::sync::UPsafeCell;
use crate::memory::address::VirtAddr;
pub struct Pid(pub usize);
pub struct PidAllocator{
    current_task_pid: usize,
    recycled: Vec<usize>
}
impl PidAllocator{
    pub fn new() -> Self {
        PidAllocator {
            current_task_pid: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> Pid {
        if let Some(pid) = self.recycled.pop() {
            Pid(pid)
        } else {
            self.current_task_pid += 1;
            Pid(self.current_task_pid - 1)
        }
    }
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current_task_pid);
        assert!(
            self.recycled.iter().find(|ppid| **ppid == pid).is_none(),
            "pid {} has been deallocated!", pid
        );
        self.recycled.push(pid);
    }
}
lazy_static! {
    static ref PID_ALLOCATOR : UPsafeCell<PidAllocator> = unsafe {
        UPsafeCell::new(PidAllocator::new())
    };
}
pub fn pid_alloc() -> Pid {
    PID_ALLOCATOR.exclusive_access().alloc()
}
impl Drop for Pid {
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}
pub struct KernelStack{
    pid: usize
}
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
impl KernelStack {
    pub fn new(pid_handle: &Pid) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        KERNEL_SPACE
            .exclusive_access()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
            );
        KernelStack {
            pid: pid_handle.0,
        }
    }
    pub fn push_on_top<T>(&self, value: T) -> *mut T where
        T: Sized, {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe { *ptr_mut = value; }
        ptr_mut
    }
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}
impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
    }
}