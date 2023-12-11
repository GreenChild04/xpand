use serde::{Serialize, Deserialize};
use crate::{version::Version, log::Log};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper {
    pub version: Version,
    pub mtype: MapperType,
    pub ids: Box<[u64]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MapperType {
    File(u64), // size in bytes
}

impl Mapper {
    #[inline]
    pub fn new(mtype: MapperType, ids: Box<[u64]>) -> Self {
        Self {
            version: Version::get_version(),
            mtype,
            ids,
        }
    }

    #[inline]
    pub fn verify_version(&self) {
        if !self.version.is_compatible(&Version::get_version()) {
            Log::Error(
                "Downloaded mapper is of an incompatible version (outdated or too new)".into(),
                Some(format!("current version is `{}` while mapper is of version `{}`", Version::get_version(), self.version))
            ).error()
        }
    }
}