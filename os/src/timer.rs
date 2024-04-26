const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1000000;
pub const MSEC_PER_SEC: usize = 1000;
use riscv::register::time;
use crate::config::qemu_CLOCK_FRED;
use crate::sbi::set_timer;

pub fn get_time()->usize{
    time::read()
}
pub fn set_next_trigger(){
    set_timer(get_time() + qemu_CLOCK_FRED / TICKS_PER_SEC);
}
pub fn get_time_us() -> usize {
    time::read() / (qemu_CLOCK_FRED / MICRO_PER_SEC)
}
pub fn get_time_ms() -> usize {
    time::read() / (qemu_CLOCK_FRED / MSEC_PER_SEC)
}
pub fn get_time_sec() -> usize {
    time::read() / qemu_CLOCK_FRED
}