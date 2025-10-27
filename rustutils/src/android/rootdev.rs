// Copyright 2025, The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module provides a function which determines the underlying block
//! device for a given file path.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use log::trace;

static POLL_DURATION: Duration = Duration::from_secs(1);

/// Error from block device operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to perform an IO operation on some block device file
    #[error("failed to perform IOs on a block device file: {0}")]
    DeviceFileIo(#[from] std::io::Error),
    /// Failed to get input file metadata
    #[error("failed to get input file metadata: {0}")]
    InputFileMetadata(std::io::Error),
    /// Block dev path has no file name component.
    #[error("block device path {0} has no file name")]
    MalformedBlockDevicePath(PathBuf),
    /// Block dev path is not valid unicode
    #[error("block device path {0} is not unicode")]
    InvalidUnicodeBlockDevicePath(PathBuf),
}

type Result<T> = std::result::Result<T, Error>;

/// For file `file_path`, retrieve the block device backing the filesystem on
/// which the file exists.
///
/// If the system is working on a snapshot merge after an OTA, the chain of dm
/// virtual block devices will terminate in a parentless dm-user device or a ublk
/// device. There doesn't appear to be a good way to resolve the underlying block
/// device from that virtual block device. When `wait_for_snapshot_merge` is set,
/// this function will poll until the snapshot merge completes. Otherwise it will
/// return the final parent dm block device.
pub fn find_backing_block_device<P: AsRef<Path> + std::fmt::Debug>(
    file_path: P,
    sysfs_path: &str,
    wait_for_snapshot_merge: bool,
) -> Result<String> {
    let start_device_name = get_block_device_name(&file_path, sysfs_path)?;
    trace!("{file_path:?} is backed by {start_device_name}");
    loop {
        let mut device_name = start_device_name.clone();
        loop {
            match get_parent_block_device(&device_name, sysfs_path)? {
                ParentBlockDeviceResult::BackingDevice => {
                    return Ok(partition_parent(&device_name, sysfs_path)?);
                }
                ParentBlockDeviceResult::SnapshotMerge => {
                    if wait_for_snapshot_merge {
                        std::thread::sleep(POLL_DURATION);
                        break;
                    } else {
                        return Ok(partition_parent(&device_name, sysfs_path)?);
                    }
                }
                ParentBlockDeviceResult::Device(parent_device) => {
                    trace!("{device_name} -> {parent_device}");
                    device_name = parent_device;
                }
            }
        }
    }
}

/// Get immediate block device name backing `file_path`.
///
/// By following the symlink `/sys/dev/block/{major}:{minor}` to the actual device path.
fn get_block_device_name<P: AsRef<Path> + std::fmt::Debug>(
    file_path: P,
    sysfs_path: &str,
) -> Result<String> {
    let devnum = fs::metadata(&file_path).map_err(Error::InputFileMetadata)?.dev() as libc::dev_t;
    // TODO: b/388993276 - Use nix::sys::stat::major|minor once they are configured to be built for Android.
    let (major, minor) = (libc::major(devnum), libc::minor(devnum));
    trace!("{file_path:?} device numbers: {major}:{minor}");
    let device_path = std::fs::canonicalize(format!("{sysfs_path}/dev/block/{major}:{minor}"))?;
    Ok(device_path
        .file_name()
        .ok_or_else(|| Error::MalformedBlockDevicePath(device_path.clone()))?
        .to_str()
        .ok_or_else(|| Error::InvalidUnicodeBlockDevicePath(device_path.clone()))?
        .to_string())
}

enum ParentBlockDeviceResult {
    // The parent device was found and has the given name.
    Device(String),
    // The device queried either has no parent (which means it is backed by
    // dm-user) or has ublk as a parent. This is a transient state while a
    // snapshot merge is occurring.
    SnapshotMerge,
    // The device queried is the final backing device.
    BackingDevice,
}

/// Returns a parent block device of a dm device with the given name.
///
/// None will be returned if:
///  * Given path doesn't correspond to a dm device.
///  * A dm device is based on top of more than one block devices.
fn get_parent_block_device(
    device_name: &str,
    sysfs_path: &str,
) -> std::io::Result<ParentBlockDeviceResult> {
    if !device_name.starts_with("dm-") {
        // Reached bottom of the device mapper stack.
        return Ok(ParentBlockDeviceResult::BackingDevice);
    }
    // Until we find a valid sub device, consider it a snapshot merge.
    let mut sub_device_name = ParentBlockDeviceResult::SnapshotMerge;
    for entry in fs::read_dir(format!("{sysfs_path}/block/{device_name}/slaves"))? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            if !matches!(sub_device_name, ParentBlockDeviceResult::SnapshotMerge) {
                // Too many slaves. Returning BackingDevice to be consistent with
                // fs_mgr's libdm implementation:
                // https://cs.android.com/android/platform/superproject/main/+/main:system/core/fs_mgr/libdm/dm.cpp;l=677-678;drc=2bd1c1b20871bcf4ef4660beaa218f2c2bce4630
                return Ok(ParentBlockDeviceResult::BackingDevice);
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("ublk") {
                sub_device_name = ParentBlockDeviceResult::Device(name)
            }
        }
    }
    Ok(sub_device_name)
}

/// Returns the parent device of a partition.
///
/// Converts e.g. "sda26" into "sda".
fn partition_parent(device_name: &str, sysfs_path: &str) -> std::io::Result<String> {
    for entry in fs::read_dir(format!("{sysfs_path}/class/block"))? {
        let name = entry?.file_name();
        let name = name.to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if fs::exists(format!("{sysfs_path}/class/block/{name}/{device_name}"))? {
            return Ok(name.to_string());
        }
    }
    trace!("Partition parent: {device_name}");
    Ok(device_name.to_string())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use tempfile::tempdir;
    use tempfile::TempDir;

    use super::*;

    enum FakeFs<'a> {
        Symlink(&'a str, &'a str),
        Dir(&'a str),
        BackingDevice(&'a Path, &'a str),
    }

    impl FakeFs<'_> {
        fn build(entries: &[Self]) -> TempDir {
            let tempdir = tempdir().unwrap();
            let root = tempdir.path();
            for entry in entries {
                match entry {
                    Self::Symlink(link, original) => {
                        Self::create_symlink(root, original, link);
                    }
                    Self::Dir(path) => {
                        fs::create_dir_all(root.join(path.trim_start_matches("/"))).unwrap();
                    }
                    Self::BackingDevice(file_path, device_path) => {
                        let device_path = root.join(device_path.trim_start_matches("/"));
                        let devnum = fs::metadata(file_path).unwrap().dev() as libc::dev_t;
                        // TODO: b/388993276 - Use nix::sys::stat::major|minor once they are configured to be built for Android.
                        let (major, minor) = (libc::major(devnum), libc::minor(devnum));
                        let link = root.join(format!("sys/dev/block/{major}:{minor}"));
                        fs::create_dir_all(link.parent().unwrap()).unwrap();
                        symlink(device_path, link).unwrap();
                    }
                }
            }
            tempdir
        }

        fn create_symlink(root: &Path, original: &str, link: &str) {
            let link = root.join(link.trim_start_matches("/"));
            let original = root.join(original.trim_start_matches("/"));
            fs::create_dir_all(link.parent().unwrap()).unwrap();
            symlink(original, link).unwrap();
        }

        fn rename(root: &Path, from: &str, to: &str) {
            let from = root.join(from.trim_start_matches("/"));
            let to = root.join(to.trim_start_matches("/"));
            std::fs::rename(from, to).unwrap();
        }
    }

    #[test]
    fn find_backing_block_device_simple() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let fake_fs = FakeFs::build(&[
            FakeFs::Dir("/sys/devices/platform/block/vda/"),
            FakeFs::Symlink("/sys/class/block/vda", "/sys/devices/platform/block/vda/"),
            FakeFs::BackingDevice(file.path(), "/sys/devices/platform/block/vda"),
        ]);

        assert_eq!(
            find_backing_block_device(
                file.path(),
                fake_fs.path().join("sys").to_str().unwrap(),
                false
            )
            .unwrap(),
            "vda"
        );
    }

    #[test]
    fn find_backing_block_device_device_mapper() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let fake_fs = FakeFs::build(&[
            FakeFs::Dir("/sys/devices/platform/block/vda/"),
            FakeFs::Dir("/sys/devices/virtual/block/dm-0/"),
            FakeFs::Dir("/sys/devices/virtual/block/dm-7/"),
            FakeFs::Symlink("/sys/block/dm-0/slaves/vda", "/sys/devices/platform/block/vda"),
            FakeFs::Symlink("/sys/block/dm-7/slaves/dm-0", "/sys/devices/virtual/block/dm-0"),
            FakeFs::Symlink("/sys/class/block/vda", "/sys/devices/platform/block/vda"),
            FakeFs::BackingDevice(file.path(), "/sys/devices/virtual/block/dm-7"),
        ]);

        assert_eq!(
            find_backing_block_device(
                file.path(),
                fake_fs.path().join("sys").to_str().unwrap(),
                false
            )
            .unwrap(),
            "vda"
        );
    }

    #[test]
    fn find_backing_block_device_ublk() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let fake_fs = FakeFs::build(&[
            FakeFs::Dir("/sys/class/block"),
            FakeFs::Dir("/sys/devices/virtual/block/ublk0/"),
            FakeFs::Dir("/sys/devices/virtual/block/dm-7/"),
            FakeFs::Symlink("/sys/block/dm-7/slaves/ublk0", "/sys/devices/virtual/block/ublk0"),
            FakeFs::BackingDevice(file.path(), "/sys/devices/virtual/block/dm-7"),
        ]);

        assert_eq!(
            find_backing_block_device(
                file.path(),
                fake_fs.path().join("sys").to_str().unwrap(),
                false
            )
            .unwrap(),
            "dm-7",
        );
    }

    #[test]
    fn find_backing_block_device_ublk_with_wait() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let fake_fs = FakeFs::build(&[
            FakeFs::Dir("/sys/devices/virtual/block/ublk0/"),
            FakeFs::Dir("/sys/devices/virtual/block/dm-7/"),
            FakeFs::Symlink("/sys/block/dm-7/slaves/ublk0", "/sys/devices/virtual/block/ublk0"),
            FakeFs::BackingDevice(file.path(), "/sys/devices/virtual/block/dm-7"),
            FakeFs::Dir("/sys/devices/virtual/block/dm-0/"),
            FakeFs::Symlink("/sys/block/dm-0/slaves/vda", "/sys/devices/platform/block/vda"),
            FakeFs::Symlink("/sys/class/block/vda", "/sys/devices/platform/block/vda"),
        ]);

        let fake_fs_path = fake_fs.path().to_path_buf();
        let test_thread = std::thread::spawn(move || {
            find_backing_block_device(file.path(), fake_fs_path.join("sys").to_str().unwrap(), true)
        });

        // We need to go through the two step rename to make sure that
        // find_backing_block_device only ever sees 1 slave.
        std::thread::sleep(POLL_DURATION * 2);
        FakeFs::create_symlink(fake_fs.path(), "/sys/devices/virtual/block/dm-0", "/sys/tmp_link");
        FakeFs::rename(fake_fs.path(), "/sys/tmp_link", "/sys/block/dm-7/slaves/ublk0");
        FakeFs::rename(
            fake_fs.path(),
            "/sys/block/dm-7/slaves/ublk0",
            "/sys/block/dm-7/slaves/dm-0",
        );

        assert_eq!(test_thread.join().unwrap().unwrap(), "vda",);
    }

    #[test]
    fn find_backing_block_device_parent_partition() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let fake_fs = FakeFs::build(&[
            FakeFs::Dir("/sys/devices/platform/block/vda/vda2"),
            FakeFs::Symlink("/sys/class/block/vda", "/sys/devices/platform/block/vda"),
            FakeFs::Symlink("/sys/class/block/vda2", "/sys/devices/platform/block/vda/vda2"),
            FakeFs::BackingDevice(file.path(), "/sys/devices/platform/block/vda/vda2"),
        ]);

        assert_eq!(
            find_backing_block_device(
                file.path(),
                fake_fs.path().join("sys").to_str().unwrap(),
                false
            )
            .unwrap(),
            "vda"
        );
    }
}
