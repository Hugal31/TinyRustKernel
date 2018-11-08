use core::mem::size_of;

use bitfield::*;

#[allow(dead_code)]
#[repr(u8)]
pub enum DPL {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

bitfield! {
    pub struct GDTEntry(u64);
    impl Debug;

    begin_segment_limit, set_segment_limit: 15, 0;
    begin_base_address, set_begin_base_address: 39, 16;
    segment_type, set_segement_type: 43, 40;
    descriptor_type, set_descriptor_type: 44;
    dpl, set_dpl: 46, 45;
    present, set_present: 47;
    end_segment_limit, set_end_segment_limit: 51, 48;
    available, set_available: 52;
    long, set_long: 53;
    db, set_db: 54;
    granularity, set_granularity: 55;
    end_base, set_end_base: 63, 56;
}

#[repr(C, packed)]
pub struct TSSEntry {
    /// The previous TSS
    pub prev_tss: u32,
    pub esp0: u32,       // The stack pointer to load when we change to kernel mode.
    pub ss0: u32,        // The stack segment to load when we change to kernel mode.
    pub esp1: u32,
    pub ss1: u32,
    pub esp2: u32,
    pub ss2: u32,
    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub es: u32,
    pub cs: u32,
    pub ss: u32,
    pub ds: u32,
    pub fs: u32,
    pub gs: u32,
    pub ldt: u32,
    pub trap: u16,
    pub iomap_base: u16,
}

impl TSSEntry {
    pub const fn new() -> TSSEntry {
        TSSEntry {
            prev_tss: 0,
            esp0: 0,
            ss0: 0,
            esp1: 0,
            ss1: 0,
            esp2: 0,
            ss2: 0,
            cr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0,
            cs: 0,
            ss: 0,
            ds: 0,
            fs: 0,
            gs: 0,
            ldt: 0,
            trap: 0,
            iomap_base: 0,
        }
    }
}

impl GDTEntry {
    const fn new_segment(base: u32, limit: u32, typ: u8, dpl: DPL, granularity: bool) -> GDTEntry {
        GDTEntry(
            limit as u64 & 0xFFFF                       // Begin limit, 16 bits
                | (base as u64 & 0xFFFFFF) << 16        // Begin base, 24 bits
                | (typ as u64) << 40                    // Segment type, 4 bits
                | 1 << 44                               // Descriptor type
                | (dpl as u64) << 45
                | 1 << 47
                | ((limit as u64 >> 16) & 0xF) << 48
                | 0 << 52
                | 0 << 53
                | 1 << 54
                | (granularity as u64) << 55
                | ((base as u64 >> 24) & 0xFF) << 56,
        )
    }

    pub const fn new_code_segment(base: u32, limit: u32, dpl: DPL, granularity: bool) -> GDTEntry {
        GDTEntry::new_segment(base, limit, 0xA, dpl, granularity)
    }

    pub const fn new_data_segment(base: u32, limit: u32, dpl: DPL, granularity: bool) -> GDTEntry {
        GDTEntry::new_segment(base, limit, 0x2, dpl, granularity)
    }

    pub const fn new_tss_segment(base: u32) -> GDTEntry {
        GDTEntry(
             size_of::<TSSEntry>() as u64 & 0xFFFF // Limit
                | (base as u64 & 0xFFFFFF) << 16        // Begin base, 24 bits
                | 0x9 << 40 // Type
                | 0x1 << 47, // Present
        )
    }
}

#[repr(C, packed)]
pub struct GDTR {
    pub limit: u16,
    pub base: u32,
}

pub fn set_lgdt(gdtr: &GDTR) {
    unsafe {
        asm!("lgdt ($0)\n\t"
             :
             : "r" (gdtr)
             : "memory");
    }
}

pub fn set_protected_mode() {
    unsafe {
        asm!("movl $$0x01, %edx
        movl %cr0, %eax
        orl %edx, %eax
        movl %eax, %cr0\n\t"
             :
             :
             : "eax", "edx"
             : "volatile");
    }
}
