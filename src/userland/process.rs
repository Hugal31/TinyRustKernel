use no_std_io::Read;
use spin::Mutex;

pub static USER_PROCESS: Mutex<Process> = Mutex::new(Process::new());

pub struct Process {
    //pub file_descriptors: [Option<&'static dyn Read + Sync>; Process::MAX_FD],
}

impl Process {
    const MAX_FD: usize = 32;

    pub const fn new() -> Process {
        Process {
            //file_descriptors: [None; Process::MAX_FD],
        }
    }
}

impl Default for Process {
    fn default() -> Self {
        Process::new()
    }
}
