use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    sup: Sup,
    program: Program,
}

#[derive(Debug, Deserialize)]
struct Sup {
    #[serde(default = "default_socket")]
    socket: String,
}

#[derive(Debug, Deserialize)]
struct Program {
    process: Process,
    log: Log,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Process {
    path: String,
    args: Vec<String>,
    #[serde(default)]
    envs: HashMap<String, String>,
    #[serde(default = "default_work_dir")]
    work_dir: String,
    #[serde(default = "default_auto_start")]
    auto_start: bool,
    #[serde(rename = "startSeconds", default = "default_start_interval")]
    start_interval: u64,
    #[serde(default = "default_restart_strategy")]
    restart_strategy: ProcessRestartStrategy,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Log {
    path: String,
    #[serde(default = "default_max_size")]
    max_size: u64,
    #[serde(default = "default_max_days")]
    max_days: u64,
    #[serde(default = "default_max_backups")]
    max_backups: u64,
    #[serde(default = "default_compress")]
    compress: bool,
    #[serde(default = "default_merge_compressed")]
    merge_compressed: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
enum ProcessRestartStrategy {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "on-failure")]
    OnFailure,
    #[serde(rename = "none")]
    AlwaysNot,
}

fn default_socket() -> String {
    String::from_str("./sup.sock").unwrap()
}

fn default_work_dir() -> String {
    String::from_str("").unwrap()
}

fn default_auto_start() -> bool {
    false
}

fn default_start_interval() -> u64 {
    5
}

fn default_restart_strategy() -> ProcessRestartStrategy {
    ProcessRestartStrategy::OnFailure
}

fn default_max_size() -> u64 {
    124217728
}

fn default_max_days() -> u64 {
    0
}

fn default_max_backups() -> u64 {
    32
}

fn default_compress() -> bool {
    false
}

fn default_merge_compressed() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_from_str() {
        let s = "[sup]
socket = \"/home/work/test/monitor/test-run/supd/run.sock\"

[program]
[program.process]
path = \"/home/work/test/monitor/test-run/conf/run.sh\"
workDir = \"/home/work/test/monitor/test-run\"
startSeconds = 5
autoStart = true
restartStrategy = \"on-failure\"

[program.log]
path = \"/home/work/test/monitor/test-run/log/run.log\"
compress = false
maxDays = 30
maxBackups = 16
maxSize = 128
";
        let t: Config = toml::from_str(s).unwrap();
        assert_eq!(
            t.program.process.restart_strategy,
            ProcessRestartStrategy::Always
        );
    }
}
