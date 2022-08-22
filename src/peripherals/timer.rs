#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use super::speaker;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Tick the clock. Should be called every hundredth of second.
// TODO Disable interrupt
pub fn tick() {
    let time = 1 + COUNTER.fetch_add(1, Ordering::AcqRel);

    speaker::tick(time * 10);
}

/// Returns the uptime in milliseconds
pub fn uptime() -> usize {
    COUNTER.load(Ordering::Relaxed) * 10
}

/// Kernel sleep. Should not be useful, we have other things to do!
pub fn sleep(milliseconds: usize) {
    let goal = COUNTER.load(Ordering::Acquire) + milliseconds / 10;
    while COUNTER.load(Ordering::Acquire) != goal {
        unsafe { llvm_asm!("hlt") };
    }
}
