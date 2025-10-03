// Copyright (C) 2025 The Android Open Source Project
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

//! Android-specific functionality

use core::ffi::c_void;
use std::convert::Infallible;

use thiserror::Error;

use crate::android::sys;

pub use crate::android::sys::{
    set_app_seccomp_filter, set_app_zygote_seccomp_filter, set_system_seccomp_filter,
};
pub use bionic_bindgen::android_fdsan_error_level as FDSanErrorLevel;

/// Errors that can occur while taking an ownership of `RawFd`
#[derive(Debug, PartialEq, Error)]
pub enum Error {
    /// Not an inherited file descriptor
    #[error("AndroidMallopt failed with opcode {0:?}")]
    MalloptFailure(MalloptOpcode),
}

/// Operations supported by [`mallopt`]
///
/// See: https://cs.android.com/android/platform/superproject/main/+/main:bionic/libc/platform/bionic/malloc.h;l=54
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum MalloptOpcode {
    /// Marks the calling process as a profileable zygote child, possibly
    /// initializing profiling infrastructure.
    InitZygoteChildProfiling = 1,
    /// Reset malloc hooks
    ResetHooks = 2,
    /// Set an upper bound on the total size in bytes of all allocations
    /// made using the memory allocation APIs.
    ///   arg = size_t*
    ///   arg_size = sizeof(size_t)
    SetAllocationLimitBytes = 3,
    /// Called after the zygote forks to indicate this is a child.
    SetZygoteChild = 4,
    /// Options to dump backtraces of allocations. These options only
    /// work when malloc debug has been enabled.
    ///
    /// Writes the backtrace information of all current allocations to a file.
    /// NOTE: arg_size has to be sizeof(FILE*) because FILE is an opaque type.
    ///   arg = FILE*
    ///   arg_size = sizeof(FILE*)
    WriteMallocLeekInfoToFile = 5,
    /// Get information about the backtraces of all
    ///   arg = android_mallopt_leak_info_t*
    ///   arg_size = sizeof(android_mallopt_leak_info_t)
    GetMallocLeakInfo = 6,
    /// Free the memory allocated and returned by M_GET_MALLOC_LEAK_INFO.
    ///   arg = android_mallopt_leak_info_t*
    ///   arg_size = sizeof(android_mallopt_leak_info_t)
    FreeMallocLeakInfo = 7,
    /// Query whether the current process is considered to be profileable by
    /// the Android platform. Result is assigned to the arg pointer's
    /// destination.
    ///   arg = bool*
    ///   arg_size = sizeof(bool)
    GetProcessProfileable = 9,
    /// Maybe enable GWP-ASan. Set *arg to force GWP-ASan to be turned on,
    /// otherwise this mallopt() will internally decide whether to sample
    /// the process. The program must be single threaded at the point when
    /// the android_mallopt function is called.
    ///   arg = android_mallopt_gwp_asan_options_t*
    ///   arg_size = sizeof(android_mallopt_gwp_asan_options_t)
    InitializeGwpAsan = 10,
    /// Query whether memtag stack is enabled for this process.
    MemtagStackIsOn = 11,
    /// Query whether the current process has the decay time enabled so
    /// that the memory from allocations are not immediately released to the
    /// OS. Result is assigned to the arg pointer's destination.
    ///   arg = bool*
    ///   arg_size = sizeof(bool)
    GetDecayTimeEnabled = 12,
}

/// A wrapper around [`sys::android_mallopt`].
///
/// # Safety
/// The user must ensure that any pointer passed into this function is of the
/// appropriate size and type for the provided opcode.
pub unsafe fn android_mallopt(
    opcode: MalloptOpcode,
    arg: *mut c_void,
    arg_size: usize,
) -> Result<(), Error> {
    // SAFETY: The caller has ensured that the argument pointer and size are
    //         valid.
    unsafe {
        bionic_bindgen::android_mallopt(opcode as _, arg, arg_size)
            .then_some(())
            .ok_or(Error::MalloptFailure(opcode))
    }
}

/// A wrapper around [`sys::android_fdsan_get_error_level`]
///
/// # Safety
/// This function is not thread safe.  Calls to this function by more than
/// one thread may result in undefined behavior.  Concurrent access must be
/// moderated by a lock or other synchronization mechanism.
///
/// See: https://android.googlesource.com/platform/bionic/+/master/docs/fdsan.md
pub unsafe fn fdsan_get_error_level() -> FDSanErrorLevel {
    // SAFETY: This function takes no arguments and always succeeds.
    unsafe { bionic_bindgen::android_fdsan_get_error_level() }
}

/// A wrapper around [`sys::android_fdsan_set_error_level`]
///
/// # Safety
/// This function is not thread safe.  Calls to this function by more than
/// one thread may result in undefined behavior.  Concurrent access must be
/// moderated by a lock or other synchronization mechanism.
///
/// See: https://android.googlesource.com/platform/bionic/+/master/docs/fdsan.md
pub unsafe fn fdsan_set_error_level(level: FDSanErrorLevel) -> FDSanErrorLevel {
    // SAFETY: This function takes an integer argument and always succeeds.
    unsafe { bionic_bindgen::android_fdsan_set_error_level(level) }
}

/// Reset the thread local stack protection salt and then call the provided
/// continuation.  It is expected that the continuation will never return.
pub fn reset_stack_guards(continuation: impl FnOnce() -> Infallible) -> Infallible {
    sys::android_reset_stack_guards();
    continuation()
}

/// A wrapper around [`libc_fill::android_mallopt`] and the M_SET_ZYGOTE_CHILD
/// opcode.
pub fn set_zygote_child() -> Result<(), Error> {
    // SAFETY: This opcode takes no arguments.
    unsafe { android_mallopt(MalloptOpcode::SetZygoteChild, std::ptr::null_mut(), 0) }
}
