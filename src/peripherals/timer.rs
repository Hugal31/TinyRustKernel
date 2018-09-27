#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Tick the clock. Should be called every hundredth of second.
pub fn tick() {
    COUNTER.fetch_add(1, Ordering::AcqRel);
}

/// Returns the uptime in milliseconds
pub fn uptime() -> usize {
    COUNTER.load(Ordering::Relaxed) * 10
}

/// Kernel sleep. Should not be useful, we have other things to do!
pub fn sleep(milliseconds: usize) {
    let goal = COUNTER.load(Ordering::Acquire) + milliseconds / 10;
    while COUNTER.load(Ordering::Acquire) != goal {
        unsafe { asm!("hlt") };
    }
}
