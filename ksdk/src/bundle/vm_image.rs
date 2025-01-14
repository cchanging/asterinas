// SPDX-License-Identifier: MPL-2.0

use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::util::hard_link_or_copy;

use super::file::BundleFile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstrosVmImage {
    path: PathBuf,
    typ: AstrosVmImageType,
    astros_version: String,
    modified_time: SystemTime,
    size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstrosVmImageType {
    GrubIso(AstrosGrubIsoImageMeta),
    Qcow2(AstrosQcow2ImageMeta),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstrosGrubIsoImageMeta {
    pub grub_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstrosQcow2ImageMeta {
    pub grub_version: String,
}

impl BundleFile for AstrosVmImage {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn modified_time(&self) -> &SystemTime {
        &self.modified_time
    }

    fn size(&self) -> &u64 {
        &self.size
    }
}

impl AstrosVmImage {
    pub fn new(path: impl AsRef<Path>, typ: AstrosVmImageType, astros_version: String) -> Self {
        let created = Self {
            path: path.as_ref().to_path_buf(),
            typ,
            astros_version,
            modified_time: SystemTime::UNIX_EPOCH,
            size: 0,
        };
        Self {
            modified_time: created.get_modified_time(),
            size: created.get_size(),
            ..created
        }
    }

    pub fn typ(&self) -> &AstrosVmImageType {
        &self.typ
    }

    /// Copy the binary to the `base` directory and convert the path to a relative path.
    pub fn copy_to(self, base: impl AsRef<Path>) -> Self {
        let file_name = self.path.file_name().unwrap();
        let copied_path = base.as_ref().join(file_name);
        hard_link_or_copy(&self.path, &copied_path).unwrap();
        let copied_metadata = copied_path.metadata().unwrap();
        Self {
            path: PathBuf::from(file_name),
            typ: self.typ,
            astros_version: self.astros_version,
            modified_time: copied_metadata.modified().unwrap(),
            size: copied_metadata.size(),
        }
    }

    pub fn astros_version(&self) -> &String {
        &self.astros_version
    }
}
