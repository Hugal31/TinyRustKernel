use core::slice;
use core::mem;

use vga;

pub const FRONTBUFFER_LEN: usize = 320 * 200 * 1;

pub fn switch_to_graphic() {
    vga::switch_mode13h()
}

pub fn swap_frontbuffer(buffer: &[u8]) -> Result<(), ()> {
    if buffer.len() != FRONTBUFFER_LEN {
        return Err(())
    }

    if let Some(ptr) = vga::get_framebuffer() {
        let frontbuffer = unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), FRONTBUFFER_LEN) };
        frontbuffer.copy_from_slice(buffer);

        Ok(())
    } else {
        Err(())
    }
}