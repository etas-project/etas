/// Only the executable invokes this, after bounded trace/profile finalization.
/// Rust's normal exit flushes stdout, whose lock may still belong to pending
/// Host I/O. No unwinding or stdio cleanup is safe once that work is abandoned.
pub(crate) fn force_process_exit(code: i32) -> ! {
    unsafe extern "C" {
        fn _exit(status: std::ffi::c_int) -> !;
    }
    // SAFETY: _exit accepts any exit status and never accesses Rust memory.
    unsafe { _exit(code) }
}
