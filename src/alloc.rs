use std::alloc::{GlobalAlloc, System};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Event {
    Alloc { addr: usize, size: usize },
    Freed { addr: usize, size: usize },
}

pub struct Tracing {
    pub inner: System,
    pub active: AtomicBool
}

impl Default for Tracing {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracing {
    pub const fn new() -> Self {
        //function can be evaluated at compile time
        Self { inner: System, active: AtomicBool::new(false) }
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    fn write_ev(&self, ev: Event) {
        let mut buf = [0u8; 1024]; //stack allocated
        let mut cursor = Cursor::new(&mut buf[..]);
        //serde_json needs to heap-allocate when serializing objects,
        let _ = serde_json::to_writer(&mut cursor, &ev);
        let end = cursor.position() as usize;
        //we need to bypass the heap allocation and use system calls to write to stderr
        self.write(&buf[..end]);
        self.write(b"\n");
    }

    fn write(&self, s: &[u8]) {
        raw_write_stderr(s);
    }
}

//used AI to write raw_write_stderr
#[cfg(windows)]
fn raw_write_stderr(s: &[u8]) {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(n_std_handle: u32) -> *mut core::ffi::c_void;
        fn WriteFile(
            h_file: *mut core::ffi::c_void,
            lp_buffer: *const u8,
            n_number_of_bytes_to_write: u32,
            lp_number_of_bytes_written: *mut u32,
            lp_overlapped: *mut core::ffi::c_void,
        ) -> i32;
    }

    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    unsafe {
        let handle = GetStdHandle(STD_ERROR_HANDLE);
        if !handle.is_null() && handle as isize != -1 {
            let mut written = 0;
            WriteFile(
                handle,
                s.as_ptr(),
                s.len() as u32,
                &mut written,
                core::ptr::null_mut(),
            );
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn raw_write_stderr(_s: &[u8]) {}

unsafe impl GlobalAlloc for Tracing {
    /// if we’re not careful, and we put some values on the heap by accident, the allocator will end up calling itself, and then we’ll have a stack overflow.
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {

        let ptr = unsafe { self.inner.alloc(layout) };
        if self.active.load(Ordering::SeqCst) {
            self.write_ev(Event::Alloc {
                addr: ptr as usize,
                size: layout.size(),
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        if self.active.load(Ordering::SeqCst) {
            self.write_ev(Event::Freed {
                addr: ptr as usize,
                size: layout.size(),
            });
        }
        unsafe { self.inner.dealloc(ptr, layout) }
    }
}

#[global_allocator] //register the type that implements GlobalAlloc as the backing engine for all dynamic allocations.
pub static ALLOCATOR: Tracing = Tracing::new(); //since this is a contsant function, we can use it as a global allocator
//GLobalAlloc specifically design for heap tracing heap allocations.