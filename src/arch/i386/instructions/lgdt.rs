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

    pub const fn new_tss_segment() -> GDTEntry {
        GDTEntry(
            0x67 // Limit
                | 0x9 << 40 // Type
                | 0x1 << 47, // Present
        )
    }
}

#[repr(C, packed)]
pub struct GDTR {
    pub limit: u16,
    pub base: *const [GDTEntry],
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
