use core::marker::PhantomData;

pub struct Port<T>
where
    T: ReadWriteFromPort,
{
    port: u16,
    _phantom: PhantomData<T>,
}

impl<T> Port<T>
where
    T: ReadWriteFromPort,
{
    pub fn new(port: u16) -> Port<T> {
        Port {
            port,
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub unsafe fn read(&mut self) -> T {
        T::read(self.port)
    }

    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        value.write(self.port)
    }
}

pub trait ReadWriteFromPort {
    unsafe fn read(port: u16) -> Self;
    unsafe fn write(self, port: u16);
}

impl ReadWriteFromPort for u8 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        let res: u8;

        asm!("inb $1, $0\n\t"
             : "={al}" (res)
             : "{dx}" (port)
             :: "volatile");

        res
    }

    #[inline]
    unsafe fn write(self, port: u16) {
        asm!("outb %al, %dx"
             :
             : "{al}" (self), "{dx}" (port)
             :: "volatile");
    }
}

impl ReadWriteFromPort for u16 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        let res: u16;

        asm!("inw $1, $0\n\t"
             : "={ax}" (res)
             : "{dx}" (port)
             :: "volatile");

        res
    }

    #[inline]
    unsafe fn write(self, port: u16) {
        asm!("outw %ax, %dx"
             :
             : "{ax}" (self), "{dx}" (port)
             :: "volatile");
    }
}
