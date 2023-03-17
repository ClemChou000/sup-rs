use std::{collections::HashMap, env, fmt::Display, fs, path::Path};

use serde::Deserialize;

use super::error;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub sup: Sup,
    pub program: Program,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Sup {
    #[serde(default = "default_socket")]
    pub socket: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Program {
    pub process: Process,
    pub log: Log,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Process {
    pub path: String,
    pub args: Option<Vec<String>>,
    pub envs: Option<HashMap<String, String>>,
    #[serde(default = "default_work_dir")]
    pub work_dir: String,
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    #[serde(rename = "startSeconds", default = "default_start_interval")]
    pub start_interval: u64,
    #[serde(default = "default_restart_strategy")]
    pub restart_strategy: ProcessRestartStrategy,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Log {
    // path is the unique identifier for process
    // rotater only handle single rotation for one path at the same time
    pub path: String,
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    #[serde(default = "default_max_days")]
    pub max_days: u64,
    #[serde(default = "default_max_backups")]
    pub max_backups: u64,
    #[serde(default = "default_compress")]
    pub compress: bool,
    #[serde(default = "default_merge_compressed")]
    pub merge_compressed: bool,
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[path:{}, max_size:{}, max_days:{}, max_backups:{}, compress:{}, merge_compressed:{}]",
            self.path,
            self.max_size,
            self.max_days,
            self.max_backups,
            self.compress,
            self.merge_compressed
        )
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ProcessRestartStrategy {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "on-failure")]
    OnFailure,
    #[serde(rename = "none")]
    AlwaysNot,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, error::Error> {
        let sr = fs::read_to_string(path);
        let mut s = String::new();
        match sr {
            Err(e) => {
                return Err(error::Error::ReadFileError(format!(
                    "read path {} failed: {}",
                    path, e
                )))
            }
            Ok(p) => s.push_str(p.as_str()),
        }

        let mut t: Config = toml::from_str(s.as_str()).unwrap();

        let work_dir_path = Path::new(&t.program.process.work_dir);
        if !work_dir_path.is_absolute() {
            return Err(error::Error::FormatCheckError(
                "work directory must be absolute".to_string(),
            ));
        }

        let path = Path::new(&t.program.process.path);
        if !path.is_absolute() {
            t.program.process.path = Path::join(work_dir_path, path)
                .to_str()
                .unwrap()
                .to_string();
        }

        let socket_path = Path::new(&t.sup.socket);
        if !socket_path.is_absolute() {
            t.sup.socket = Path::join(work_dir_path, socket_path)
                .to_str()
                .unwrap()
                .to_string();
        }

        let logp = Path::new(&t.program.log.path);
        if !logp.is_absolute() {
            t.program.log.path = Path::join(work_dir_path, logp)
                .to_str()
                .unwrap()
                .to_string();
        }

        Ok(t)
    }
}

fn default_socket() -> String {
    "./sup.sock".to_string()
}

fn default_work_dir() -> String {
    env::current_dir().unwrap().to_str().unwrap().to_string()
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
            t,
            Config {
                sup: Sup {
                    socket: "/home/work/test/monitor/test-run/supd/run.sock".to_string()
                },
                program: Program {
                    process: Process {
                        path: "/home/work/test/monitor/test-run/conf/run.sh".to_string(),
                        args: None,
                        envs: None,
                        work_dir: "/home/work/test/monitor/test-run".to_string(),
                        auto_start: true,
                        start_interval: 5,
                        restart_strategy: ProcessRestartStrategy::OnFailure,
                    },
                    log: Log {
                        path: "/home/work/test/monitor/test-run/log/run.log".to_string(),
                        max_size: 128,
                        max_days: 30,
                        max_backups: 16,
                        compress: false,
                        merge_compressed: false
                    }
                }
            }
        );
    }

    #[test]
    fn read_from_file() {
        let path = "../../test/config/config.toml";
        match Config::new(path) {
            Ok(c) => {
                assert_eq!(
                    c,
                    Config {
                        sup: Sup {
                            socket: "/home/work/test/monitor/test-run/supd/run.sock".to_string()
                        },
                        program: Program {
                            process: Process {
                                path: "/home/work/test/monitor/test-run/conf/run.sh".to_string(),
                                args: None,
                                envs: None,
                                work_dir: "/home/work/test/monitor/test-run".to_string(),
                                auto_start: true,
                                start_interval: 5,
                                restart_strategy: ProcessRestartStrategy::OnFailure,
                            },
                            log: Log {
                                path: "/home/work/test/monitor/test-run/log/run.log".to_string(),
                                max_size: 128,
                                max_days: 30,
                                max_backups: 16,
                                compress: false,
                                merge_compressed: false,
                            }
                        }
                    }
                )
            }
            Err(_) => {}
        }
    }
}
