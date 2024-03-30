use core::panic::PanicInfo;

#[panic_handler]
fn paninc_handler(panic_info: &PanicInfo)->!{
    loop {

    }
}