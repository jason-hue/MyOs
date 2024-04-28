mod virtio_blk;
mod block_device;

pub use virtio_blk::VirtIOBlock;
pub use block_device::BlockDevice; //这里从easyfs替换为同一目录下的Blockevice,也要给其他文件用
use crate::board::BlockDeviceImpl;
use alloc::sync::Arc;

use lazy_static::*;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}
