use riscv::register::sstatus::{self, Sstatus, SPP};
pub struct TrapContext{
    pub x: [usize;32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,//kernel's token
    pub kernel_sp: usize,
    pub trap_handler: usize,
}
impl TrapContext{
    fn set_sp(&mut self, sp: usize){
        self.x[2] = sp;
    }
    pub fn init_trap_context(entry_pc: usize, sp: usize, kernel_satp: usize, kernel_sp: usize, trap_handler: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self{
            x: [0;32],
            sstatus,
            sepc: entry_pc,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }

}