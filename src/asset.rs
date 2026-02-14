use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum AssetLocation {
    Local(PathBuf),
    Remote {
        url: String,
        checksum: Option<String>,
    },
}

impl AssetLocation {
    pub fn get_local_path(&self) -> Option<PathBuf> {
        match self {
            AssetLocation::Local(path) => Some(path.clone()),
            AssetLocation::Remote { url, checksum } => {
                // 逻辑：检查本地缓存文件夹是否已经有这个文件
                // 如果没有，返回 None 或触发下载流
                todo!("实现缓存查找逻辑")
            }
        }
    }
}
#[derive(Debug, Deserialize, Clone)]
pub struct SongAsset {
    pub audio: AssetLocation,
}

#[derive(Debug, Deserialize)]
pub struct ChartAsset {
    pub chart_file: AssetLocation,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IlluAsset {
    pub illu: AssetLocation,
}