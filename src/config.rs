use crate::core::judge::JudgeCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::error;

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub song_dir_path: String,
    pub log_path: String,
    pub poll_period: i32,
    pub playing: PlayingConfig
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayingConfig {
    pub global_offset_ms: i32,
    pub ready_seconds: f64, // 正值
    pub judge_core: JudgeCore,
    pub keybind: HashMap<char, u8>,
    pub show_potential_acc: bool,
    pub show_potential_rank: bool,
    pub show_debug_overlay: bool,
}

impl GlobalConfig {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}
pub fn json_to_config(json_str: &str) -> anyhow::Result<GlobalConfig> {
    let config = serde_json::from_str(json_str).inspect_err(
        |e| error!("Error parsing config: {e}")
    )?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::config::{GlobalConfig, PlayingConfig};
    use crate::core::judge::{JudgeCore, JudgeWindow};
    use crate::core::timing::Time;
    use super::*;
    #[test]
    fn test_gen_config() {
        let keybind = HashMap::from(
            [
                ('d', 0),
                ('f', 1),
                ('j', 2),
                ('k', 3),
            ]
        );
        let judge_core = JudgeCore{
            window: JudgeWindow {
                perfect: Time(0.08),
                good: Time(0.16),
            },
            hold_tolerance: Time(0.008)
        };
        let config: GlobalConfig = GlobalConfig {
            song_dir_path: "./assets".into(),
            poll_period: 4,
            log_path: "./game.log".into(),
            playing: PlayingConfig {
                global_offset_ms: 800,
                ready_seconds: 3.0,
                show_potential_acc: true,
                show_potential_rank: true,
                show_debug_overlay: true,
                keybind,
                judge_core
            }
        };

        println!("{}", config.to_json().unwrap());
    }
}