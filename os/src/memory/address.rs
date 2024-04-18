use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct PhysAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct VirtAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct PhysPageNum(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct VirtPageNum(pub usize);
const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;//44位字节地址
const VA_WIDTH_SV39: usize = 39;
const VPN_WIDTH_SV39: usize = VPN_WIDTH_SV39 - PAGE_SIZE_BITS;//27位
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << PA_WIDTH_SV39) - 1 )) }
}//将usize类型转换为PhysAddr类型。在转换过程中，它将输入的usize值按位与操作符(&)与((1 << PA_WIDTH_SV39) - 1)进行按位与操作，从而保留了usize值的低PA_WIDTH_SV39位，将高位清零，得到一个合法的物理地址
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self { Self(v & ( (1 << PPN_WIDTH_SV39) - 1 )) }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { v.0 }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << VA_WIDTH_SV39) - 1 )) }
}
impl From<usize> for VirtPageNum{
    fn from(v: usize) -> Self { Self(v & ( (1 << VPN_WIDTH_SV39) - 1 )) }
}
impl From<VirtAddr> for usize{
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}
impl From<VirtPageNum> for usize{
    fn from(value: VirtPageNum) -> Self {
        value.0
    }
}
impl PhysAddr{
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }//得到物理地址的低12位
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }//大于4096为第一个页面，0～4096为第0个页面，物理地址除页面大小是为了取整求这是第几个页面
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}
/*
PhysAddr = PPN + page_offset
VirtAddr = VPN + page_offset
*/
impl VirtAddr{
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE -1)
    }
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE -1) / PAGE_SIZE)
    }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<PhysPageNum> for PhysAddr{
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}