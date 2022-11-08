use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::Deserialize;

use super::error;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    sup: Sup,
    program: Program,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Sup {
    #[serde(default = "default_socket")]
    socket: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Program {
    process: Process,
    log: Log,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Process {
    path: PathBuf,
    args: Option<Vec<String>>,
    envs: Option<HashMap<String, String>>,
    #[serde(default = "default_work_dir")]
    work_dir: PathBuf,
    #[serde(default = "default_auto_start")]
    auto_start: bool,
    #[serde(rename = "startSeconds", default = "default_start_interval")]
    start_interval: u64,
    #[serde(default = "default_restart_strategy")]
    restart_strategy: ProcessRestartStrategy,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Log {
    path: PathBuf,
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

        let work_dir = &t.program.process.work_dir;
        if !work_dir.is_absolute() {
            return Err(error::Error::FormatCheckError(
                "work directory must be absolute".to_string(),
            ));
        }

        let path = &t.program.process.path;
        if !path.is_absolute() {
            t.program.process.path = Path::join(work_dir.as_path(), path);
        }

        let socket = &t.sup.socket;
        if !socket.is_absolute() {
            t.sup.socket = Path::join(work_dir.as_path(), socket);
        }

        let logp = &t.program.log.path;
        if !logp.is_absolute() {
            t.program.log.path = Path::join(work_dir.as_path(), logp);
        }

        Ok(t)
    }
}

fn default_socket() -> PathBuf {
    PathBuf::from_str("./sup.sock").unwrap()
}

fn default_work_dir() -> PathBuf {
    env::current_dir().unwrap()
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
                    socket: PathBuf::from_str("/home/work/test/monitor/test-run/supd/run.sock")
                        .unwrap()
                },
                program: Program {
                    process: Process {
                        path: PathBuf::from_str("/home/work/test/monitor/test-run/conf/run.sh")
                            .unwrap(),
                        args: None,
                        envs: None,
                        work_dir: PathBuf::from_str("/home/work/test/monitor/test-run").unwrap(),
                        auto_start: true,
                        start_interval: 5,
                        restart_strategy: ProcessRestartStrategy::OnFailure,
                    },
                    log: Log {
                        path: PathBuf::from_str("/home/work/test/monitor/test-run/log/run.log")
                            .unwrap(),
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
                            socket: PathBuf::from_str(
                                "/home/work/test/monitor/test-run/supd/run.sock"
                            )
                            .unwrap()
                        },
                        program: Program {
                            process: Process {
                                path: PathBuf::from_str(
                                    "/home/work/test/monitor/test-run/conf/run.sh"
                                )
                                .unwrap(),
                                args: None,
                                envs: None,
                                work_dir: PathBuf::from_str("/home/work/test/monitor/test-run")
                                    .unwrap(),
                                auto_start: true,
                                start_interval: 5,
                                restart_strategy: ProcessRestartStrategy::OnFailure,
                            },
                            log: Log {
                                path: PathBuf::from_str(
                                    "/home/work/test/monitor/test-run/log/run.log"
                                )
                                .unwrap(),
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
