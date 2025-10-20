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

//! This module provides external definitions for BIONIC-specific functionality
//!
//! Mangled C++ names provided to `link_name` attributes were suggested by the
//! linker when no name was provided.  The format for functions that take no
//! parameters and return no values is:
//! `'_Z' + Min API Level + Function Name + 'v'`.

// Temporarily allow dead code as the rest of the module is defined in
// following commits.
#[allow(dead_code)]
unsafe extern "C" {
    /// Reset the thread's stack protection salt
    ///
    /// The caller should not return after calling this function.  If it does,
    /// and stack protection is enabled, the program will crash.
    ///
    /// See: https://cs.android.com/android/platform/superproject/main/+/main:bionic/libc/bionic/__libc_init_main_thread.cpp;l=104
    pub(crate) safe fn android_reset_stack_guards();

    /// Apply Android's application seccomp filters
    #[link_name = "_Z22set_app_seccomp_filterv"]
    pub safe fn set_app_seccomp_filter();

    /// Apply Android's app zygote seccomp filters
    #[link_name = "_Z29set_app_zygote_seccomp_filterv"]
    pub safe fn set_app_zygote_seccomp_filter();

    /// Apply Android's system seccomp filters
    #[link_name = "_Z25set_system_seccomp_filterv"]
    pub safe fn set_system_seccomp_filter();

    /// Apply a seccomp filter that limits setresuid/setresgid to a range of
    /// [uid_gid_min..uid_gid_max] (for the real-, effective- and super-ids).
    #[link_name = "_Z32install_setuidgid_seccomp_filterjj"]
    pub safe fn install_setuidgid_seccomp_filter(uid_gid_min: u32, uid_gid_max: u32);
}
