//! Raw declarations for the contract in `include/dmd_core.h`.
//!
//! Callers must obey that header's pointer, lifetime, thread and state requirements.

pub const DMD_CORE_ABI_VERSION: u32 = 1;
pub const DMD_OK: i32 = 0;
pub const DMD_END: i32 = 1;
pub const DMD_ERROR: i32 = -1;
pub const DMD_FORMAT_REAL_F32: u32 = 1;
pub const DMD_FORMAT_IQ_F32: u32 = 2;
pub const DMD_TEXT_INFO: u32 = 1;
pub const DMD_TEXT_LIST: u32 = 2;
pub const DMD_TEXT_MASK: u32 = 3;
pub const DMD_DIAG_VERSION_CONFLICT: u32 = 1;
pub const DMD_DIAG_AMBIGUOUS_NAME: u32 = 2;

#[repr(C)]
pub struct DmdSignal {
    _private: [u8; 0],
}

#[repr(C)]
pub struct DmdCatalogue {
    _private: [u8; 0],
}

#[repr(C)]
pub struct DmdRun {
    _private: [u8; 0],
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdBytes {
    pub data: *const u8,
    pub len: u64,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdError {
    pub code: i32,
    pub message: DmdBytes,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdStream {
    pub format: u32,
    pub channels: u32,
    pub rate_numerator: u64,
    pub rate_denominator: u64,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdBlock {
    pub samples: *const f32,
    pub frames: u64,
    pub source_position: u64,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdModule {
    pub id: DmdBytes,
    pub name: DmdBytes,
    pub version: DmdBytes,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdDiagnostic {
    pub kind: u32,
    pub name: DmdBytes,
    pub message: DmdBytes,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdParameter {
    pub name: DmdBytes,
    pub value: DmdBytes,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct DmdResult {
    pub identifier: DmdBytes,
    pub payload: DmdBytes,
}

unsafe extern "C" {
    pub fn dmd_core_abi_version() -> u32;
    pub fn dmd_signal_open(path: DmdBytes, out: *mut *mut DmdSignal, error: *mut DmdError) -> i32;
    pub fn dmd_signal_close(signal: *mut DmdSignal);
    pub fn dmd_signal_source(
        signal: *mut DmdSignal,
        out: *mut DmdStream,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_signal_target(
        signal: *mut DmdSignal,
        target: *const DmdStream,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_signal_output(
        signal: *mut DmdSignal,
        out: *mut DmdStream,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_signal_read(signal: *mut DmdSignal, out: *mut DmdBlock, error: *mut DmdError)
    -> i32;
    pub fn dmd_signal_finish(signal: *mut DmdSignal, error: *mut DmdError) -> i32;
    pub fn dmd_signal_drain(
        signal: *mut DmdSignal,
        out: *mut DmdBlock,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_catalogue_open(
        paths: *const DmdBytes,
        count: u64,
        out: *mut *mut DmdCatalogue,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_catalogue_close(catalogue: *mut DmdCatalogue);
    pub fn dmd_catalogue_module(
        catalogue: *mut DmdCatalogue,
        index: u64,
        out: *mut DmdModule,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_catalogue_text(
        catalogue: *mut DmdCatalogue,
        kind: u32,
        id: DmdBytes,
        out: *mut DmdBytes,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_catalogue_diagnostic(
        catalogue: *mut DmdCatalogue,
        index: u64,
        out: *mut DmdDiagnostic,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_run_create(
        catalogue: *mut DmdCatalogue,
        id: DmdBytes,
        parameters: *const DmdParameter,
        count: u64,
        stream: *const DmdStream,
        out: *mut *mut DmdRun,
        error: *mut DmdError,
    ) -> i32;
    pub fn dmd_run_submit(run: *mut DmdRun, block: *const DmdBlock, error: *mut DmdError) -> i32;
    pub fn dmd_run_next(run: *mut DmdRun, out: *mut DmdResult, error: *mut DmdError) -> i32;
    pub fn dmd_run_finalize(run: *mut DmdRun, error: *mut DmdError) -> i32;
    pub fn dmd_run_destroy(run: *mut DmdRun);
}
