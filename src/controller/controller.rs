use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Ok, Result};
use tokio::process::Command;

use crate::config::config::{Process, ProcessRestartStrategy};

use super::command::Command as mCommand;

pub struct ProcessController {
    exec_status: AtomicUsize, // 0 ==> not running 1 ==> running
    cmd: Command,
    args: Option<Vec<String>>,
    dir: String,
    env: Option<HashMap<String, String>>,
    interval: u64,
    restart: ProcessRestartStrategy,
}

impl ProcessController {
    pub async fn new(conf: Process) -> Result<Self> {
        let pc = Self {
            exec_status: AtomicUsize::new(0),
            cmd: Command::new(conf.path),
            args: conf.args,
            dir: conf.work_dir,
            env: conf.envs,
            interval: conf.start_interval,
            restart: conf.restart_strategy,
        };
        if conf.auto_start {
            Self::start_cmd(&pc).await?
        }
        Ok(pc)
    }

    pub async fn exec_cmd(&self, cmd: mCommand) -> Result<()> {
        if self.is_executing() {
            return Ok(());
        }

        self.start_cmd().await;
        self.set_executing();
        Ok(())
    }

    fn is_executing(&self) -> bool {
        // TODO: change ordering to relaxed?
        !self
            .exec_status
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    fn set_executing(&self) {
        self.exec_status.store(1, Ordering::SeqCst)
    }

    async fn start_cmd(&self) -> Result<()> {
        if self.is_running().await? {
            return Ok(());
        }

        Ok(())
    }

    async fn is_running(&self) -> Result<bool> {
        Ok(false)
    }
}
