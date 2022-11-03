use bus::Bus;
use crossbeam_utils::sync::WaitGroup;

const MAX_RUNNING_THREADS: usize = 1000;

type RunFunc = Box<dyn FnOnce() + Send + 'static>;

trait Run {
    fn stop(&mut self);
    fn wait(self);
    fn stop_and_wait(self);
}

struct Runner {
    done: WaitGroup,
    bus: Bus<()>,
}

impl Runner {
    pub fn new(rfs: Vec<RunFunc>) -> Option<Self> {
        if rfs.len() == 0 {
            return None;
        }
        let bus = Bus::new(rfs.len());
        for rf in rfs {}
        Some(Self {
            done: WaitGroup::new(),
            bus,
        })
    }
}

impl Run for Runner {
    fn stop(&mut self) {
        self.bus.broadcast(());
    }

    fn wait(self) {
        self.done.wait();
    }

    fn stop_and_wait(mut self) {
        self.stop();
        self.wait();
    }
}
