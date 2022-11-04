use log::{debug, error};
use std::thread::{self, JoinHandle};

use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    select,
};

use super::error::RunError;

type Task = Box<dyn FnOnce() + Send + 'static>;

type Opt = Box<dyn FnOnce(&mut ThreadsPool) + Send + 'static>;

enum Msg {
    Bye,
    T(Task),
}

fn with_num(num: usize) -> Opt {
    return Box::new(move |tp: &mut ThreadsPool| {
        tp.num = num;
    });
}

struct Work {
    jh: Option<JoinHandle<()>>,
    alias: String,
}

pub struct ThreadsPool {
    num: usize,
    threads: Vec<Work>,

    unbounded_task_recv: Receiver<Msg>,
    unbounded_task_send: Sender<Msg>,
}

impl ThreadsPool {
    pub fn new(opts: Vec<Opt>) -> Self {
        let (us, ur) = unbounded();
        let mut slf = Self {
            num: num_cpus::get(),
            threads: Vec::<Work>::new(),

            unbounded_task_recv: ur,
            unbounded_task_send: us,
        };
        for opt in opts {
            opt(&mut slf)
        }
        Self::run(&mut slf);
        slf
    }

    pub fn add_task(&self, task: Task) -> Result<(), RunError> {
        if let Err(e) = self.unbounded_task_send.send(Msg::T(task)) {
            return Err(RunError::SendTaskFail(e.to_string()));
        };
        Ok(())
    }

    fn run(&mut self) {
        for i in 0..self.num {
            let r = self.unbounded_task_recv.clone();
            let t = thread::spawn(move || loop {
                select! {
                    recv(r) -> msg => {
                        match msg{
                            Ok(t) => {
                                match t {
                                    Msg::Bye => break,
                                    Msg::T(t) => t(),
                                }
                            },
                            Err(e) => error!("recv task err:{}",e),
                        }
                    }
                }
            });
            self.threads.push(Work {
                jh: Some(t),
                alias: String::from(format!("thread{}", i)),
            });
        }
    }
}
impl Drop for ThreadsPool {
    fn drop(&mut self) {
        for _ in 0..self.num {
            self.unbounded_task_send.send(Msg::Bye).unwrap();
        }
        for j in self.threads.iter_mut() {
            if let Some(t) = j.jh.take() {
                t.join().unwrap();
                debug!("{} in threadpool finished", j.alias);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_num() {
        env_logger::init();
        let tp = ThreadsPool::new(vec![with_num(3)]);
        tp.add_task(Box::new(|| {
            println!("111");
        }))
        .unwrap();
        tp.add_task(Box::new(|| {
            println!("222");
        }))
        .unwrap();
        tp.add_task(Box::new(|| {
            println!("333");
        }))
        .unwrap();
    }
}
