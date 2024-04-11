use riscv::register::sstatus::{self, Sstatus, SPP};
pub struct TrapContext{
    pub x: [usize;32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}
impl TrapContext{
    fn set_sp(&mut self, sp: usize){
        self.x[2] = sp;
    }
    pub fn init_trap_context(entry_pc: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self{
            x: [0;32],
            sstatus,
            sepc: entry_pc,
        };
        cx.set_sp(sp);
        cx
    }

}