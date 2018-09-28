#![allow(dead_code)]

use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::Mutex;

use super::timer;

use crate::arch::i386::instructions::Port;
use crate::arch::i386::pit::PIT;

static CURRENT_MELODY: Mutex<Option<Melody>> = Mutex::new(None);
static CURRENT_TONE_END_DATE: AtomicUsize = AtomicUsize::new(0);

/// Represent a  not in the melody
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Tone {
    /// Frequency in hertz
    pub frequency: u32,
    /// Duration in milliseconds
    pub duration: u32,
}

impl Tone {
    pub const fn new(frequency: u32, duration: u32) -> Self {
        Tone {
            frequency,
            duration,
        }
    }

    fn is_end(&self) -> bool {
        self.frequency == 0
    }
}

pub fn play_frequency(frequency: u32) {
    PIT.lock().play_sound(frequency);
}

pub fn start_melody_from(melody: *const Tone, repeating: bool) {
    let mut length = 0;
    let mut cursor = melody;
    let s = unsafe {
        while !(&*cursor).is_end() {
            length += 1;
            cursor = cursor.add(1);
        }

        slice::from_raw_parts(melody, length)
    };
    start_melody(s, repeating);
}

// TODO Find other thing than 'static
pub fn start_melody(melody: &'static [Tone], repeating: bool) {
    let mut current = CURRENT_MELODY.lock();
    let mut melody = Melody::new(melody, repeating);
    if let Some(tone) = melody.next() {
        play_tone(tone);
    }

    current.replace(melody);

    enable();
}

/// To call every hundredth of second
pub fn tick(time: usize) {
    let end_date = CURRENT_TONE_END_DATE.load(Ordering::Acquire);
    if end_date != 0 && end_date <= time {
        next_tone();
    }
}

fn next_tone() {
    if let Some(tone) = {
        let mut current = CURRENT_MELODY.lock();
        (*current).as_mut().and_then(|m| m.next().cloned()).clone()
    } {
        play_tone(&tone);
    } else {
        CURRENT_MELODY.lock().take();
        CURRENT_TONE_END_DATE.store(0, Ordering::SeqCst);
        disable();
    }
}

fn play_tone(tone: &Tone) {
    play_frequency(tone.frequency);
    CURRENT_TONE_END_DATE.store(timer::uptime() + tone.duration as usize, Ordering::SeqCst);
}

pub fn enable() {
    unsafe {
        // FIXME: This is architecture dependent and should be in arch module
        let mut port = Port::<u8>::new(0x61);
        let tmp = port.read();
        port.write(tmp | 3);
    }
}

pub fn disable() {
    unsafe {
        // FIXME: This is architecture dependent and should be in arch module
        let mut port = Port::<u8>::new(0x61);
        let tmp = port.read();
        port.write(tmp & !3);
    }
}

struct Melody<'a> {
    tones: &'a [Tone],
    index: usize,
    repeating: bool,
}

impl<'a> Melody<'a> {
    pub fn new(tones: &'a [Tone], repeating: bool) -> Self {
        Melody {
            tones,
            index: 0,
            repeating,
        }
    }
}

impl<'a> Iterator for Melody<'a> {
    type Item = &'a Tone;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tones.len() {
            if self.repeating {
                self.index = 0
            } else {
                return None;
            }
        }

        let tone = &self.tones[self.index];
        self.index += 1;
        Some(tone)
    }
}
