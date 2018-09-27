use core::ops::Add;

use spin::Mutex;

static COUNTER: Mutex<usize> = Mutex::new(0);

pub fn tick() {
    let mut counter = COUNTER.lock();
    *counter = counter.add(1);
}
