use crate::{utils::get_core, synchronization::{SpinLock, interface::Mutex}};
use alloc::boxed::Box;

#[link_section = ".locks"]
pub static PTABLE: PTable = PTable::new();

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
struct CPUContext {
    x19: u64,
    x20: u64,
    x21: u64,
    x22: u64,
    x23: u64,
    x24: u64,
    x25: u64,
    x26: u64,
    x27: u64,
    x28: u64,
    fp: u64,
    sp: u64,
    pc: u64,
}

impl CPUContext {
    const fn empty() -> Self {
        Self {
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            fp: 0,
            sp: 0,
            pc: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum PState {
    TaskUnused,
    TaskSleep,
    TaskRunning,
    TaskZombie,
}


#[repr(align(16))]
//#[derive(Copy, Clone)]
#[derive(Debug)]
struct Process {
    ctx: CPUContext,
    state: PState,
    name: &'static str,
    pid: usize,
    core_using: u8,
    next: Option<Box<Process>>,
}

impl Process {
    const fn empty() -> Self {
        Self {
            ctx: CPUContext::empty(),
            state: PState::TaskUnused,
            name: "",
            pid: !0,
            core_using: !0,
            next: None
        }
    }
}

pub struct PTable {
    inner: SpinLock<PTableInner>,
}

impl PTable {
    pub const fn new() -> Self {
        Self {
            inner: SpinLock::new(PTableInner::new()),
        }
    }

    pub fn init_core(&self) {
        let mut table = self.inner.lock().unwrap();
        table.init_core_inner(get_core());
    }

    /*pub fn create_proc(&self, f: fn()) {
        let mut table = self.inner.lock().unwrap();
        table.create_proc_inner();
    }*/

    pub fn print(&self) {
        let table = self.inner.lock().unwrap();
        table.print();
    }
}

struct PTableInner {
    num_procs: usize,
    head: Option<Box<Process>>,
    running: [Option<Box<Process>>; 4],
}

impl PTableInner {
    const fn new() -> Self {
        Self {
            num_procs: 0,
            head: None,
            running: [None, None, None, None],
        }
    }

    fn init_core_inner(&mut self, core: u8) {
        let init_proc = Box::new(Process {
            ctx: CPUContext::empty(),
            state: PState::TaskRunning,
            name: "kthread",
            pid: self.num_procs + 1,
            core_using: core,
            next: None,
        });
        self.running[core as usize] = Some(init_proc);
        self.num_procs += 1;
    }

    /*fn create_proc_inner(&mut self, f: fn()) {
        let new_proc = Box::new(Process {
            ctx: CPUContext::new_process(f),
            state: todo!(),
            name: todo!(),
            pid: todo!(),
            core_using: todo!(),
            next: todo!(),
        });
    }*/

    fn print(&self) {
        for i in 0..4 {
            if let Some(curproc) = &self.running[i] {
                crate::println!("{:?}", curproc);
            }
        }
        let cur = &self.head;
        while let Some(curproc) = cur {
            crate::println!("{:#?}", curproc);
        }
    }
}