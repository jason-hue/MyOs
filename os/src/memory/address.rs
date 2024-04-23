use core::fmt;
use core::fmt::{Debug, Formatter};
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use crate::memory::page_table::PageTableEntry;

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
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;//27位
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << PA_WIDTH_SV39) - 1 )) }
}
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
impl PhysPageNum{
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8,4096)
        }
    }
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa:PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry,512)
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
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
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl PhysAddr {
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize;3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }// vpn2 = idx[0]   vpn1 = idx[1]    vpn0 = idx[2]

}
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}
pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}
#[derive(Copy, Clone, Debug)]
pub struct SimpleRange<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd + Debug
{
    left: T,
    right: T,
}

impl<T> SimpleRange<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd + Debug
{
    pub fn new(start: T,end: T)->Self{
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self{
            left: start,
            right: end,
        }
    }
    pub fn get_start(&self)->T{
        self.left
    }
    pub fn get_end(&self)->T{
        self.right
    }
}

impl<T> IntoIterator for SimpleRange<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd + Debug
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.left, self.right)
    }
}
pub struct SimpleRangeIterator<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
pub type VPNRange = SimpleRange<VirtPageNum>;