// Only a u8 port for now
pub struct Port {
    port: u16,
}

impl Port {
    pub const fn new(port: u16) -> Port {
        Port { port }
    }

    #[allow(dead_code)]
    #[inline]
    pub unsafe fn read(&mut self) -> u8 {
        let res: u8;

        asm!("inb $1, $0\n\t"
             : "=a" (res)
             : "d" (self.port)
             :: "volatile");

        res
    }

    #[inline]
    pub unsafe fn write(&mut self, value: u8) {
        asm!("outb %al, %dx"
             :
             : "{ax}" (value), "{dx}" (self.port)
             :: "volatile");
    }
}
