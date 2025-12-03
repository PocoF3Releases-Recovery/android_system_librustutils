//
// Copyright (C) 2025 The Android Open-Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Safe wrappers around functions exported by libcap
//!
//! To build with cargo, you need to install libcap-dev

// The `bitflags!` macro generate an associated constant that we can't
// document.
#![allow(missing_docs)]

use bitflags::bitflags;
use nix::errno::Errno;
use thiserror::Error;

use cap_bindgen::{__user_cap_data_struct, __user_cap_header_struct, cap_value_t, CAP_LAST_CAP};

type CapDataPair = [__user_cap_data_struct; 2];

const CAP_DATA_EMPTY: [__user_cap_data_struct; 2] =
    [__user_cap_data_struct { effective: 0, permitted: 0, inheritable: 0 }; 2];
const CAP_HEADER_V3: __user_cap_header_struct = cap_header(CapabilityVersion::V3, 0);

#[derive(Error, Debug)]
pub enum CapabilityError {
    #[error("Failed to clear capabilities flag: {0:?}")]
    ClearFlagFailure(Errno),
    #[error("Failed to drop capabilities: {0:?}")]
    DropFailure(Errno),
    #[error("Failed to get capabilities: {0:?}")]
    GetCapFailure(Errno),
    #[error("Failed to get capability bound: {0:?}")]
    GetCapBoundFailure(Errno),
    #[error("Failed to set capabilities: {0:?}")]
    SetCapFailure(Errno),
    #[error("Failed to set process capabilities: {0:?}")]
    SetProcFailure(Errno),
    #[error("Invalid capability value: {0:?}")]
    InvalidCapability(u32),
}

type Result<T> = std::result::Result<T, CapabilityError>;

/// A representation of *capabilities* (not _flags_)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum Capability {
    CHOWN = cap_bindgen::CAP_CHOWN,
    DAC_OVERRIDE = cap_bindgen::CAP_DAC_OVERRIDE,
    DAC_READ_SEARCH = cap_bindgen::CAP_DAC_READ_SEARCH,
    FOWNER = cap_bindgen::CAP_FOWNER,
    FSETID = cap_bindgen::CAP_FSETID,
    KILL = cap_bindgen::CAP_KILL,
    SETGID = cap_bindgen::CAP_SETGID,
    SETUID = cap_bindgen::CAP_SETUID,
    SETPCAP = cap_bindgen::CAP_SETPCAP,
    LINUX_IMMUTABLE = cap_bindgen::CAP_LINUX_IMMUTABLE,
    NET_BIND_SERVICE = cap_bindgen::CAP_NET_BIND_SERVICE,
    NET_BROADCAST = cap_bindgen::CAP_NET_BROADCAST,
    NET_ADMIN = cap_bindgen::CAP_NET_ADMIN,
    NET_RAW = cap_bindgen::CAP_NET_RAW,
    IPC_LOCK = cap_bindgen::CAP_IPC_LOCK,
    IPC_OWNER = cap_bindgen::CAP_IPC_OWNER,
    SYS_MODULE = cap_bindgen::CAP_SYS_MODULE,
    SYS_RAWIO = cap_bindgen::CAP_SYS_RAWIO,
    SYS_CHROOT = cap_bindgen::CAP_SYS_CHROOT,
    SYS_PTRACE = cap_bindgen::CAP_SYS_PTRACE,
    SYS_PACCT = cap_bindgen::CAP_SYS_PACCT,
    SYS_ADMIN = cap_bindgen::CAP_SYS_ADMIN,
    SYS_BOOT = cap_bindgen::CAP_SYS_BOOT,
    SYS_NICE = cap_bindgen::CAP_SYS_NICE,
    SYS_RESOURCE = cap_bindgen::CAP_SYS_RESOURCE,
    SYS_TIME = cap_bindgen::CAP_SYS_TIME,
    SYS_TTY_CONFIG = cap_bindgen::CAP_SYS_TTY_CONFIG,
    MKNOD = cap_bindgen::CAP_MKNOD,
    LEASE = cap_bindgen::CAP_LEASE,
    AUDIT_WRITE = cap_bindgen::CAP_AUDIT_WRITE,
    AUDIT_CONTROL = cap_bindgen::CAP_AUDIT_CONTROL,
    SETFCAP = cap_bindgen::CAP_SETFCAP,
    MAC_OVERRIDE = cap_bindgen::CAP_MAC_OVERRIDE,
    MAC_ADMIN = cap_bindgen::CAP_MAC_ADMIN,
    SYSLOG = cap_bindgen::CAP_SYSLOG,
    WAKE_ALARM = cap_bindgen::CAP_WAKE_ALARM,
    BLOCK_SUSPEND = cap_bindgen::CAP_BLOCK_SUSPEND,
    AUDIT_READ = cap_bindgen::CAP_AUDIT_READ,
    PERFMON = cap_bindgen::CAP_PERFMON,
    BPF = cap_bindgen::CAP_BPF,
    CHECKPOINT_RESTORE = cap_bindgen::CAP_CHECKPOINT_RESTORE,
}

impl TryFrom<u32> for Capability {
    type Error = CapabilityError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            cap_bindgen::CAP_CHOWN => Ok(Capability::CHOWN),
            cap_bindgen::CAP_DAC_OVERRIDE => Ok(Capability::DAC_OVERRIDE),
            cap_bindgen::CAP_DAC_READ_SEARCH => Ok(Capability::DAC_READ_SEARCH),
            cap_bindgen::CAP_FOWNER => Ok(Capability::FOWNER),
            cap_bindgen::CAP_FSETID => Ok(Capability::FSETID),
            cap_bindgen::CAP_KILL => Ok(Capability::KILL),
            cap_bindgen::CAP_SETGID => Ok(Capability::SETGID),
            cap_bindgen::CAP_SETUID => Ok(Capability::SETUID),
            cap_bindgen::CAP_SETPCAP => Ok(Capability::SETPCAP),
            cap_bindgen::CAP_LINUX_IMMUTABLE => Ok(Capability::LINUX_IMMUTABLE),
            cap_bindgen::CAP_NET_BIND_SERVICE => Ok(Capability::NET_BIND_SERVICE),
            cap_bindgen::CAP_NET_BROADCAST => Ok(Capability::NET_BROADCAST),
            cap_bindgen::CAP_NET_ADMIN => Ok(Capability::NET_ADMIN),
            cap_bindgen::CAP_NET_RAW => Ok(Capability::NET_RAW),
            cap_bindgen::CAP_IPC_LOCK => Ok(Capability::IPC_LOCK),
            cap_bindgen::CAP_IPC_OWNER => Ok(Capability::IPC_OWNER),
            cap_bindgen::CAP_SYS_MODULE => Ok(Capability::SYS_MODULE),
            cap_bindgen::CAP_SYS_RAWIO => Ok(Capability::SYS_RAWIO),
            cap_bindgen::CAP_SYS_CHROOT => Ok(Capability::SYS_CHROOT),
            cap_bindgen::CAP_SYS_PTRACE => Ok(Capability::SYS_PTRACE),
            cap_bindgen::CAP_SYS_PACCT => Ok(Capability::SYS_PACCT),
            cap_bindgen::CAP_SYS_ADMIN => Ok(Capability::SYS_ADMIN),
            cap_bindgen::CAP_SYS_BOOT => Ok(Capability::SYS_BOOT),
            cap_bindgen::CAP_SYS_NICE => Ok(Capability::SYS_NICE),
            cap_bindgen::CAP_SYS_RESOURCE => Ok(Capability::SYS_RESOURCE),
            cap_bindgen::CAP_SYS_TIME => Ok(Capability::SYS_TIME),
            cap_bindgen::CAP_SYS_TTY_CONFIG => Ok(Capability::SYS_TTY_CONFIG),
            cap_bindgen::CAP_MKNOD => Ok(Capability::MKNOD),
            cap_bindgen::CAP_LEASE => Ok(Capability::LEASE),
            cap_bindgen::CAP_AUDIT_WRITE => Ok(Capability::AUDIT_WRITE),
            cap_bindgen::CAP_AUDIT_CONTROL => Ok(Capability::AUDIT_CONTROL),
            cap_bindgen::CAP_SETFCAP => Ok(Capability::SETFCAP),
            cap_bindgen::CAP_MAC_OVERRIDE => Ok(Capability::MAC_OVERRIDE),
            cap_bindgen::CAP_MAC_ADMIN => Ok(Capability::MAC_ADMIN),
            cap_bindgen::CAP_SYSLOG => Ok(Capability::SYSLOG),
            cap_bindgen::CAP_WAKE_ALARM => Ok(Capability::WAKE_ALARM),
            cap_bindgen::CAP_BLOCK_SUSPEND => Ok(Capability::BLOCK_SUSPEND),
            cap_bindgen::CAP_AUDIT_READ => Ok(Capability::AUDIT_READ),
            cap_bindgen::CAP_PERFMON => Ok(Capability::PERFMON),
            cap_bindgen::CAP_BPF => Ok(Capability::BPF),
            cap_bindgen::CAP_CHECKPOINT_RESTORE => Ok(Capability::CHECKPOINT_RESTORE),
            _ => Err(CapabilityError::InvalidCapability(value)),
        }
    }
}

/// The storage type for capability flags
pub type RawCap = u64;

bitflags! {
    /// Bitflags representing Linux capabilities
    ///
    /// See: `man capabilities`
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CapabilityFlags: RawCap {
        const CHOWN = 1 << cap_bindgen::CAP_CHOWN;
        const DAC_OVERRIDE = 1 << cap_bindgen::CAP_DAC_OVERRIDE;
        const DAC_READ_SEARCH = 1 << cap_bindgen::CAP_DAC_READ_SEARCH;
        const FOWNER = 1 << cap_bindgen::CAP_FOWNER;
        const FSETID = 1 << cap_bindgen::CAP_FSETID;
        const KILL = 1 << cap_bindgen::CAP_KILL;
        const SETGID = 1 << cap_bindgen::CAP_SETGID;
        const SETUID = 1 << cap_bindgen::CAP_SETUID;
        const SETPCAP = 1 << cap_bindgen::CAP_SETPCAP;
        const LINUX_IMMUTABLE = 1 << cap_bindgen::CAP_LINUX_IMMUTABLE;
        const NET_BIND_SERVICE = 1 << cap_bindgen::CAP_NET_BIND_SERVICE;
        const NET_BROADCAST = 1 << cap_bindgen::CAP_NET_BROADCAST;
        const NET_ADMIN = 1 << cap_bindgen::CAP_NET_ADMIN;
        const NET_RAW = 1 << cap_bindgen::CAP_NET_RAW;
        const IPC_LOCK = 1 << cap_bindgen::CAP_IPC_LOCK;
        const IPC_OWNER = 1 << cap_bindgen::CAP_IPC_OWNER;
        const SYS_MODULE = 1 << cap_bindgen::CAP_SYS_MODULE;
        const SYS_RAWIO = 1 << cap_bindgen::CAP_SYS_RAWIO;
        const SYS_CHROOT = 1 << cap_bindgen::CAP_SYS_CHROOT;
        const SYS_PTRACE = 1 << cap_bindgen::CAP_SYS_PTRACE;
        const SYS_PACCT = 1 << cap_bindgen::CAP_SYS_PACCT;
        const SYS_ADMIN = 1 << cap_bindgen::CAP_SYS_ADMIN;
        const SYS_BOOT = 1 << cap_bindgen::CAP_SYS_BOOT;
        const SYS_NICE = 1 << cap_bindgen::CAP_SYS_NICE;
        const SYS_RESOURCE = 1 << cap_bindgen::CAP_SYS_RESOURCE;
        const SYS_TIME = 1 << cap_bindgen::CAP_SYS_TIME;
        const SYS_TTY_CONFIG = 1 << cap_bindgen::CAP_SYS_TTY_CONFIG;
        const MKNOD = 1 << cap_bindgen::CAP_MKNOD;
        const LEASE = 1 << cap_bindgen::CAP_LEASE;
        const AUDIT_WRITE = 1 << cap_bindgen::CAP_AUDIT_WRITE;
        const AUDIT_CONTROL = 1 << cap_bindgen::CAP_AUDIT_CONTROL;
        const SETFCAP = 1 << cap_bindgen::CAP_SETFCAP;
        const MAC_OVERRIDE = 1 << cap_bindgen::CAP_MAC_OVERRIDE;
        const MAC_ADMIN = 1 << cap_bindgen::CAP_MAC_ADMIN;
        const SYSLOG = 1 << cap_bindgen::CAP_SYSLOG;
        const WAKE_ALARM = 1 << cap_bindgen::CAP_WAKE_ALARM;
        const BLOCK_SUSPEND = 1 << cap_bindgen::CAP_BLOCK_SUSPEND;
        const AUDIT_READ = 1 << cap_bindgen::CAP_AUDIT_READ;
        const PERFMON = 1 << cap_bindgen::CAP_PERFMON;
        const BPF = 1 << cap_bindgen::CAP_BPF;
        const CHECKPOINT_RESTORE = 1 << cap_bindgen::CAP_CHECKPOINT_RESTORE;
    }
}

/// A struct containing effective, permitted, and
/// inheritable permissions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilitiesSet {
    effective: CapabilityFlags,
    permitted: CapabilityFlags,
    inheritable: CapabilityFlags,
}

impl CapabilitiesSet {
    /*
     * Constructors
     */

    /// Creates a new [`CapabilitiesSet`] with all flags set to zero
    pub const fn const_default() -> Self {
        Self {
            effective: CapabilityFlags::empty(),
            permitted: CapabilityFlags::empty(),
            inheritable: CapabilityFlags::empty(),
        }
    }

    /// Construct a [`CapabilitiesSet`] from a pair of `libcap` structs
    fn from_cap_data_pair(data: &CapDataPair) -> Self {
        Self {
            effective: CapabilityFlags::from_bits_truncate(
                data[0].effective as RawCap | ((data[1].effective as RawCap) << 32),
            ),
            permitted: CapabilityFlags::from_bits_truncate(
                data[0].permitted as RawCap | ((data[1].permitted as RawCap) << 32),
            ),
            inheritable: CapabilityFlags::from_bits_truncate(
                data[0].inheritable as RawCap | ((data[1].inheritable as RawCap) << 32),
            ),
        }
    }

    /// Creates a new [`CapabilitiesSet`] from the raw bitmasks
    pub fn from_raw(effective: RawCap, permitted: RawCap, inheritable: RawCap) -> Self {
        Self {
            effective: CapabilityFlags::from_bits_truncate(effective),
            permitted: CapabilityFlags::from_bits_truncate(permitted),
            inheritable: CapabilityFlags::from_bits_truncate(inheritable),
        }
    }

    /// A safe wrapper around the `libcap` implementation of `capget`
    ///
    /// See: `man capget`
    pub fn load() -> Result<Self> {
        let mut header = CAP_HEADER_V3;
        let mut data = CAP_DATA_EMPTY;

        // SAFETY: The pointers passed to this function are guaranteed to
        //         be valid as they refer to regions allocated on the local
        //         stack frame. The return value is checked and wrapped in
        //         a `LibcResult`.
        let result =
            unsafe { cap_bindgen::capget(std::ptr::addr_of_mut!(header), data.as_mut_ptr()) };

        if result == 0 {
            Ok(Self::from_cap_data_pair(&data))
        } else {
            Err(CapabilityError::GetCapFailure(Errno::last()))
        }
    }

    /// The base constructor for [`CapabilitiesSet`]
    pub fn new(
        effective: CapabilityFlags,
        permitted: CapabilityFlags,
        inheritable: CapabilityFlags,
    ) -> Self {
        Self { effective, permitted, inheritable }
    }

    /// Build a new [`CapabilitiesSet`], replacing any capability classes if
    /// [`Some`] values are provided
    pub fn replace_some(
        &self,
        effective: Option<CapabilityFlags>,
        permitted: Option<CapabilityFlags>,
        inheritable: Option<CapabilityFlags>,
    ) -> Self {
        *self.clone().overwrite_some(effective, permitted, inheritable)
    }

    /// Creates a new [`CapabilitiesSet`] by combining the flags of `self` and
    /// `other`.
    pub fn with(&self, other: &Self) -> Self {
        Self {
            effective: self.effective | other.effective,
            permitted: self.permitted | other.permitted,
            inheritable: self.inheritable | other.inheritable,
        }
    }

    /// Build a new [`CapabilitiesSet`], adding any new capabilities if
    /// provided with [`Some`] values
    pub fn with_some(
        &self,
        effective: Option<CapabilityFlags>,
        permitted: Option<CapabilityFlags>,
        inheritable: Option<CapabilityFlags>,
    ) -> Self {
        *self.clone().add_some(effective, permitted, inheritable)
    }

    /// Return a new [`CapabilitiesSet`] with the provided flags added to the
    /// effective set
    pub fn with_effective(&self, flags: CapabilityFlags) -> Self {
        Self { effective: self.effective | flags, ..*self }
    }

    /// Return a new [`CapabilitiesSet`] with the provided flags added to the
    /// permitted set
    pub fn with_permitted(&self, flags: CapabilityFlags) -> Self {
        Self { permitted: self.permitted | flags, ..*self }
    }

    /// Return a new [`CapabilitiesSet`] with the provided flags added to the
    /// inheritable set
    pub fn with_inheritable(&self, flags: CapabilityFlags) -> Self {
        Self { inheritable: self.inheritable | flags, ..*self }
    }

    /// Return a new [`CapabilitiesSet`] without the provided flags added to
    /// wrapped capabilities sets
    pub fn without(&self, other: &Self) -> Self {
        Self {
            effective: self.effective & !other.effective,
            permitted: self.permitted & !other.permitted,
            inheritable: self.inheritable & !other.inheritable,
        }
    }

    /// Return a new [`CapabilitiesSet`] without the provided flags included in
    /// the effective set
    pub fn without_effective(&self, flags: CapabilityFlags) -> Self {
        Self { effective: self.effective & !flags, ..*self }
    }

    /// Return a new [`CapabilitiesSet`] without the provided flags included in
    /// the permitted set
    pub fn without_permitted(&self, flags: CapabilityFlags) -> Self {
        Self { permitted: self.permitted & !flags, ..*self }
    }

    /// Return a new [`CapabilitiesSet`] without the provided flags included in
    /// the permitted set
    pub fn without_inheritable(&self, flags: CapabilityFlags) -> Self {
        Self { inheritable: self.inheritable & !flags, ..*self }
    }

    /*
     * Kernel Stores
     */

    /// A safe wrapper around the `libcap` implementation of `capset`.  This
    /// function will read the existing capabilities, add this struct's
    /// capabilities, and pass the resulting data to the kernel.
    ///
    /// See: `man capset`
    pub fn store_additive(&self) -> Result<()> {
        Self::load()?.add(self).store_overwrite()
    }

    /// A safe wrapper around the `libcap` implementation of `capset`.  This
    /// function will pass the struct's data to the kernel and overwrite the
    /// existing capabilities.
    ///
    /// See: `man capset`
    pub fn store_overwrite(&self) -> Result<()> {
        let mut header = CAP_HEADER_V3;
        let mut data = self.as_user_cap_data();

        // SAFETY: The pointers passed to this function are guaranteed to be
        //         valid as they refer to regions allocated on the local stack
        //         frame. The return value is checked and wrapped in a
        //         `LibcResult`.
        let result =
            unsafe { cap_bindgen::capset(std::ptr::addr_of_mut!(header), data.as_mut_ptr()) };

        if result == 0 {
            Ok(())
        } else {
            Err(CapabilityError::SetCapFailure(Errno::last()))
        }
    }

    /*
     * Getters and Setters
     */

    /// Returns the effective capabilities
    pub fn effective(&self) -> CapabilityFlags {
        self.effective
    }

    /// Returns the permitted capabilities
    pub fn permitted(&self) -> CapabilityFlags {
        self.permitted
    }

    /// Returns the inheritable capabilities
    pub fn inheritable(&self) -> CapabilityFlags {
        self.inheritable
    }

    /// Adds the flags of `other` to this struct's bitmasks
    pub fn add(&mut self, other: &Self) -> &mut Self {
        self.effective |= other.effective;
        self.permitted |= other.permitted;
        self.inheritable |= other.inheritable;

        self
    }

    /// Add capabilities to classes when provided with [`Some`] values
    pub fn add_some(
        &mut self,
        effective: Option<CapabilityFlags>,
        permitted: Option<CapabilityFlags>,
        inheritable: Option<CapabilityFlags>,
    ) -> &mut Self {
        if let Some(caps) = effective {
            self.effective |= caps;
        }
        if let Some(caps) = permitted {
            self.permitted |= caps;
        }
        if let Some(caps) = inheritable {
            self.inheritable |= caps;
        }

        self
    }

    /// Add the provided flags to the effective set
    pub fn add_effective(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.effective |= flags;

        self
    }

    /// Add the provided flags to the permitted set
    pub fn add_permitted(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.permitted |= flags;

        self
    }

    /// Add the provided flags to the inheritable set
    pub fn add_inheritable(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.inheritable |= flags;

        self
    }

    /// Marshal the data into `libcap` data structures
    fn as_user_cap_data(&self) -> CapDataPair {
        [
            __user_cap_data_struct {
                effective: self.effective.bits() as u32,
                permitted: self.permitted.bits() as u32,
                inheritable: self.inheritable.bits() as u32,
            },
            __user_cap_data_struct {
                effective: (self.effective.bits() >> 32) as u32,
                permitted: (self.permitted.bits() >> 32) as u32,
                inheritable: (self.inheritable.bits() >> 32) as u32,
            },
        ]
    }

    /// Test to see if the provided flags are in the effective capabilities
    pub fn has_effective(&self, flags: CapabilityFlags) -> bool {
        self.effective.contains(flags)
    }

    /// Test to see if the provided flags are in the permitted capabilities
    pub fn has_permitted(&self, flags: CapabilityFlags) -> bool {
        self.permitted.contains(flags)
    }

    /// Test to see if the provided flags are in the inheritable capabilities
    pub fn has_inheritable(&self, flags: CapabilityFlags) -> bool {
        self.inheritable.contains(flags)
    }

    /// Replace existing capability classes when provided with [`Some`] values
    pub fn overwrite_some(
        &mut self,
        effective: Option<CapabilityFlags>,
        permitted: Option<CapabilityFlags>,
        inheritable: Option<CapabilityFlags>,
    ) -> &mut Self {
        if let Some(caps) = effective {
            self.effective = caps;
        }
        if let Some(caps) = permitted {
            self.permitted = caps;
        }
        if let Some(caps) = inheritable {
            self.inheritable = caps;
        }

        self
    }

    /// Removes the flags of `other` to this struct's bitmasks
    pub fn remove(&mut self, other: &Self) -> &mut Self {
        self.effective &= !other.effective;
        self.permitted &= !other.permitted;
        self.inheritable &= !other.inheritable;

        self
    }

    /// Remove the provided flags from the effective set
    pub fn remove_effective(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.effective &= !flags;

        self
    }

    /// Remove the provided flags from the permitted set
    pub fn remove_permitted(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.permitted &= !flags;

        self
    }

    /// Remove the provided flags from the inheritable set
    pub fn remove_inheritable(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.inheritable &= !flags;

        self
    }

    /// Set the effective capabilities
    pub fn set_effective(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.effective = flags;

        self
    }

    /// Set the permitted capabilities
    pub fn set_permitted(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.permitted = flags;

        self
    }

    /// Set the inheritable capabilities
    pub fn set_inheritable(&mut self, flags: CapabilityFlags) -> &mut Self {
        self.inheritable = flags;

        self
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CapabilityVersion {
    V1,
    V2,
    V3,
}

impl CapabilityVersion {
    pub const fn value(&self) -> u32 {
        match self {
            CapabilityVersion::V1 => cap_bindgen::_LINUX_CAPABILITY_VERSION_1,
            CapabilityVersion::V2 => cap_bindgen::_LINUX_CAPABILITY_VERSION_2,
            CapabilityVersion::V3 => cap_bindgen::_LINUX_CAPABILITY_VERSION_3,
        }
    }
}

const fn cap_header(version: CapabilityVersion, pid: i32) -> __user_cap_header_struct {
    __user_cap_header_struct { version: version.value(), pid }
}

/// Drop a bounding capability.  This capability can never be re-gained
pub fn cap_drop_bound(cap: Capability) -> Result<()> {
    // SAFETY: This function takes no pointers and return value is checked and
    //         wrapped in a `Result` type.
    if unsafe { cap_bindgen::cap_drop_bound(cap as cap_bindgen::cap_value_t) } == 0 {
        Ok(())
    } else {
        Err(CapabilityError::DropFailure(Errno::last()))
    }
}

/// Test to see if a capability is within the bounding set
pub fn cap_within_bound(cap: Capability) -> Result<bool> {
    // SAFETY: This function takes no pointers and return value is checked and
    //         wrapped in a `Result<bool>` type.
    match unsafe { cap_bindgen::cap_get_bound(cap as cap_bindgen::cap_value_t) } {
        -1 => Err(CapabilityError::GetCapBoundFailure(Errno::last())),
        ret => Ok(ret > 0),
    }
}

/// Returns the maximum capability bit that is defined in `libcap`.  If the
/// kernel returns a capability with a higher value then we need to update
/// `libcap`.
///
/// TODO: Implement tests to check when either this library or libcap are out
///       of date.
pub fn cap_max_bits() -> cap_value_t {
    // SAFETY: This function takes no arguments and can't fail.
    unsafe { cap_bindgen::cap_max_bits() }
}

/// Removes inheritable capabilities set for this process.
///
/// See: https://man7.org/linux/man-pages/man7/capabilities.7.html
pub fn drop_inheritable_caps() -> Result<()> {
    // SAFETY: we do not manipulate memory handled by libcap.
    unsafe {
        let caps = cap_bindgen::cap_get_proc();
        scopeguard::defer! {
            cap_bindgen::cap_free(caps as *mut std::os::raw::c_void);
        }

        if cap_bindgen::cap_clear_flag(caps, cap_bindgen::cap_flag_t::CAP_INHERITABLE) < 0 {
            Err(CapabilityError::ClearFlagFailure(Errno::last()))
        } else if cap_bindgen::cap_set_proc(caps) < 0 {
            Err(CapabilityError::SetProcFailure(Errno::last()))
        } else {
            Ok(())
        }
    }
}

/// Drop bounding set capabilities for this process.
///
/// See: https://man7.org/linux/man-pages/man7/capabilities.7.html
pub fn drop_bounding_set() -> Result<()> {
    for cap_id in 0u32..=CAP_LAST_CAP {
        cap_drop_bound(Capability::try_from(cap_id)?)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    // Basic test to verify that calling drop_inheritable_caps doesn't fail
    #[test]
    fn test_drop_inheritable_caps() {
        let result = super::drop_inheritable_caps();
        assert!(result.is_ok(), "failed with: {result:?}")
    }
}
