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

//! A crate to use malloc/free/etc. as your allocator without `libstd`.
//! This is suitable if your environment has libc, but does not have libstd available.
//!
//! You cannot just link this crate in. You must also use `extern crate` to force the dependency.

#![no_std]

use core::alloc::{GlobalAlloc, Layout};

unsafe extern "C" {
    safe fn aligned_alloc(align: usize, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

struct Malloc;

#[global_allocator]
static MALLOC: Malloc = Malloc;

// SAFETY: This impl forwards to the libc implementation of an allocator, which matches Rust's
// expectations.
unsafe impl GlobalAlloc for Malloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        aligned_alloc(layout.align(), layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY: by trait preconditions, this pointer came from alloc/realloc and hasn't been
        // released yet.
        unsafe { free(ptr) };
    }
}
