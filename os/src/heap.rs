use crate::linear_allocator::LinearAllocator;

#[global_allocator]
static mut KERNEL_HEAP_ALLOCATOR: LinearAllocator = LinearAllocator::empty();
static mut KERNEL_HEAP: [u8;0x20000] = [0;0x20000];

pub unsafe fn init_kernel_heap(){
    let heap_start = KERNEL_HEAP.as_ptr() as usize;
    let heap_size = KERNEL_HEAP.len();
    KERNEL_HEAP_ALLOCATOR.init(heap_start,heap_size);
}