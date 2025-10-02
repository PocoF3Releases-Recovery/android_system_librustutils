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

//! Provides a small implementation of an `abort` panic strategy, usable in environments which
//! have a `bionic` libc, but may not have `libstd` available. If you have `libstd`, prefer that.
//!
//! You cannot just add this crate as a dependency. You must also reference it explicitly through
//! `extern crate`, or the panic handler may not be set appropriately.

#![no_std]

use core::fmt::Write;
use core::panic::PanicInfo;

unsafe extern "C" {
    fn android_set_abort_message(msg: *const core::ffi::c_char);
    safe fn abort() -> !;
}

struct PanicFormatter {
    buf: [u8; 1024],
    write_head: usize,
}

impl PanicFormatter {
    fn new() -> Self {
        Self { buf: [0; 1024], write_head: 0 }
    }
}

impl Write for PanicFormatter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if s.len() < self.buf.len() - self.write_head {
            self.buf[self.write_head..self.write_head + s.len()].copy_from_slice(s.as_bytes());
            self.write_head += s.len();
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut fmt = PanicFormatter::new();
    if write!(fmt, "{}\0", info).is_ok() {
        // SAFETY: If write worked, fmt.buf is a NUL-terminated buffer, which will live
        // across the call.
        unsafe { android_set_abort_message(fmt.buf.as_ptr() as _) }
    }
    abort();
}
