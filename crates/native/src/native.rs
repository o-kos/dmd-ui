use std::{marker::PhantomData, ptr::NonNull, rc::Rc};

use dmd_core_sys as sys;

use crate::Error;

/// A validated sample layout and rational frame rate.
#[derive(Clone, Copy, Debug)]
pub struct Stream(sys::DmdStream);

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Real,
    Iq,
}

impl Stream {
    pub fn new(
        format: Format,
        channels: u32,
        numerator: u64,
        denominator: u64,
    ) -> Result<Self, Error> {
        Self::validate(sys::DmdStream {
            format: match format {
                Format::Real => sys::DMD_FORMAT_REAL_F32,
                Format::Iq => sys::DMD_FORMAT_IQ_F32,
            },
            channels,
            rate_numerator: numerator,
            rate_denominator: denominator,
        })
    }

    fn validate(raw: sys::DmdStream) -> Result<Self, Error> {
        if !matches!(
            raw.format,
            sys::DMD_FORMAT_REAL_F32 | sys::DMD_FORMAT_IQ_F32
        ) || raw.channels == 0
            || raw.rate_numerator == 0
            || raw.rate_denominator == 0
        {
            return Err(Error::Input("invalid stream characteristics"));
        }
        Ok(Self(raw))
    }

    pub fn characteristics(&self) -> (Format, u32, u64, u64) {
        let format = if self.0.format == sys::DMD_FORMAT_IQ_F32 {
            Format::Iq
        } else {
            Format::Real
        };
        (
            format,
            self.0.channels,
            self.0.rate_numerator,
            self.0.rate_denominator,
        )
    }

    fn scalars(&self, frames: u64) -> Result<usize, Error> {
        let components = if self.0.format == sys::DMD_FORMAT_IQ_F32 {
            2
        } else {
            1
        };
        frames
            .checked_mul(u64::from(self.0.channels))
            .and_then(|n| n.checked_mul(components))
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| *n <= isize::MAX as usize / size_of::<f32>())
            .ok_or(Error::Input("sample count is too large"))
    }
}

/// An owned prepared block; position is measured in source frames.
#[derive(Debug)]
pub struct Block {
    pub samples: Vec<f32>,
    pub frames: u64,
    pub source_position: u64,
}

fn status(code: i32) -> Result<(), Error> {
    if code == sys::DMD_OK {
        Ok(())
    } else {
        Err(Error::Contract("unexpected operation status"))
    }
}

// The callback must return an ABI status and diagnostic, keeping its span live
// until this returns. Each production callback makes exactly one core call.
unsafe fn call(
    operation: &'static str,
    invoke: impl FnOnce(&mut sys::DmdError) -> i32,
) -> Result<i32, Error> {
    let mut error = sys::DmdError::default();
    let code = invoke(&mut error);
    if code != sys::DMD_ERROR {
        return Ok(code);
    }
    // SAFETY: The caller guarantees the diagnostic remains live. Copy before another
    // core call; the ABI owns this span and forbids callers from freeing it.
    let message = unsafe { copy_text(error.message) }?;
    Err(Error::Core {
        operation,
        code: error.code,
        message,
    })
}

fn bytes(value: &str) -> sys::DmdBytes {
    sys::DmdBytes {
        data: value.as_ptr(),
        len: value.len() as u64,
    }
}

fn check_version() -> Result<(), Error> {
    // SAFETY: The version query has no preconditions or resources.
    if unsafe { sys::dmd_core_abi_version() } != sys::DMD_CORE_ABI_VERSION {
        return Err(Error::Contract("unsupported C ABI version"));
    }
    Ok(())
}

// SAFETY requirements are discharged at each call: the core returned this span
// from a successful operation and no subsequent operation has invalidated it.
unsafe fn copy_bytes(value: sys::DmdBytes) -> Result<Vec<u8>, Error> {
    if value.len == 0 {
        return Ok(Vec::new());
    }
    let length = usize::try_from(value.len)
        .ok()
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(Error::Contract("invalid byte length"))?;
    if value.data.is_null() {
        return Err(Error::Contract("null byte span"));
    }
    // SAFETY: The caller guarantees the returned span remains valid; bounds were checked.
    Ok(unsafe { std::slice::from_raw_parts(value.data, length) }.to_vec())
}

unsafe fn copy_text(value: sys::DmdBytes) -> Result<String, Error> {
    // SAFETY: The caller provides the same live returned span required by copy_bytes.
    String::from_utf8(unsafe { copy_bytes(value) }?).map_err(|_| Error::Contract("invalid UTF-8"))
}

/// A signal handle, confined to the thread that opened it.
pub struct Signal {
    handle: NonNull<sys::DmdSignal>,
    thread: PhantomData<Rc<()>>,
    started: bool,
    finished: bool,
}

impl Signal {
    pub fn open(path: &str) -> Result<Self, Error> {
        check_version()?;
        if path.contains('\0') {
            return Err(Error::Input("path contains NUL"));
        }
        let mut out = std::ptr::null_mut();
        // SAFETY: Path and output storage remain valid for this synchronous call.
        status(unsafe {
            call("dmd_signal_open", |error| {
                sys::dmd_signal_open(bytes(path), &mut out, error)
            })
        }?)?;
        Ok(Self {
            handle: NonNull::new(out).ok_or(Error::Contract("null signal"))?,
            thread: PhantomData,
            started: false,
            finished: false,
        })
    }

    pub fn source(&mut self) -> Result<Stream, Error> {
        let mut out = sys::DmdStream::default();
        // SAFETY: The exclusively borrowed handle is live and output storage is valid.
        status(unsafe {
            call("dmd_signal_source", |error| {
                sys::dmd_signal_source(self.handle.as_ptr(), &mut out, error)
            })
        }?)?;
        Stream::validate(out)
    }

    pub fn output(&mut self) -> Result<Stream, Error> {
        let mut out = sys::DmdStream::default();
        // SAFETY: The exclusively borrowed handle is live and output storage is valid.
        status(unsafe {
            call("dmd_signal_output", |error| {
                sys::dmd_signal_output(self.handle.as_ptr(), &mut out, error)
            })
        }?)?;
        Stream::validate(out)
    }

    pub fn set_target(&mut self, target: Stream) -> Result<(), Error> {
        if self.started || self.finished {
            return Err(Error::Input("target must precede reading"));
        }
        // SAFETY: The handle is live, the target validated, and reading has not begun.
        status(unsafe {
            call("dmd_signal_target", |error| {
                sys::dmd_signal_target(self.handle.as_ptr(), &target.0, error)
            })
        }?)
    }

    pub fn read(&mut self) -> Result<Option<Block>, Error> {
        if self.finished {
            return Err(Error::Input("signal is finished"));
        }
        self.started = true;
        self.block(false)
    }

    pub fn finish(&mut self) -> Result<(), Error> {
        if self.finished {
            return Err(Error::Input("signal is already finished"));
        }
        self.finished = true;
        // SAFETY: The live handle has not previously been finished.
        status(unsafe {
            call("dmd_signal_finish", |error| {
                sys::dmd_signal_finish(self.handle.as_ptr(), error)
            })
        }?)
    }

    pub fn drain(&mut self) -> Result<Option<Block>, Error> {
        if !self.finished {
            return Err(Error::Input("finish before draining"));
        }
        self.block(true)
    }

    fn block(&mut self, drain: bool) -> Result<Option<Block>, Error> {
        let stream = self.output()?;
        let mut out = sys::DmdBlock::default();
        // SAFETY: The handle is live, state checked by the caller, and output storage is valid.
        let code = unsafe {
            if drain {
                call("dmd_signal_drain", |error| {
                    sys::dmd_signal_drain(self.handle.as_ptr(), &mut out, error)
                })
            } else {
                call("dmd_signal_read", |error| {
                    sys::dmd_signal_read(self.handle.as_ptr(), &mut out, error)
                })
            }
        }?;
        if code == sys::DMD_END {
            return Ok(None);
        }
        status(code)?;
        let length = stream.scalars(out.frames)?;
        if length == 0 || out.samples.is_null() {
            return Err(Error::Contract("empty prepared block"));
        }
        // SAFETY: The ABI promises this many samples until the next handle call; length is checked.
        let samples = unsafe { std::slice::from_raw_parts(out.samples, length) }.to_vec();
        Ok(Some(Block {
            samples,
            frames: out.frames,
            source_position: out.source_position,
        }))
    }
}

impl Drop for Signal {
    fn drop(&mut self) {
        // SAFETY: This owner closes its live handle exactly once on its creating thread.
        unsafe { sys::dmd_signal_close(self.handle.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub version: String,
}
#[derive(Debug)]
pub struct Diagnostic {
    pub kind: u32,
    pub name: String,
    pub message: String,
}
#[derive(Clone, Copy)]
pub enum Text {
    Info,
    List,
    Mask,
}

pub struct Catalogue {
    handle: NonNull<sys::DmdCatalogue>,
    thread: PhantomData<Rc<()>>,
}

impl Catalogue {
    pub fn open(paths: &[&str]) -> Result<Self, Error> {
        check_version()?;
        if paths.iter().any(|path| path.contains('\0')) {
            return Err(Error::Input("path contains NUL"));
        }
        let paths: Vec<_> = paths.iter().map(|path| bytes(path)).collect();
        let mut out = std::ptr::null_mut();
        // SAFETY: The path spans and output storage are live for the duration of the call.
        status(unsafe {
            call("dmd_catalogue_open", |error| {
                sys::dmd_catalogue_open(paths.as_ptr(), paths.len() as u64, &mut out, error)
            })
        }?)?;
        Ok(Self {
            handle: NonNull::new(out).ok_or(Error::Contract("null catalogue"))?,
            thread: PhantomData,
        })
    }

    pub fn module(&mut self, index: u64) -> Result<Option<Module>, Error> {
        let mut out = sys::DmdModule::default();
        // SAFETY: The live handle is exclusively borrowed and output storage is valid.
        let code = unsafe {
            call("dmd_catalogue_module", |error| {
                sys::dmd_catalogue_module(self.handle.as_ptr(), index, &mut out, error)
            })
        }?;
        if code == sys::DMD_END {
            return Ok(None);
        }
        status(code)?;
        // SAFETY: All spans come from this successful call, with no intervening handle calls.
        unsafe {
            Ok(Some(Module {
                id: copy_text(out.id)?,
                name: copy_text(out.name)?,
                version: copy_text(out.version)?,
            }))
        }
    }

    pub fn text(&mut self, kind: Text, id: &str) -> Result<String, Error> {
        let kind = match kind {
            Text::Info => sys::DMD_TEXT_INFO,
            Text::List => sys::DMD_TEXT_LIST,
            Text::Mask => sys::DMD_TEXT_MASK,
        };
        let mut out = sys::DmdBytes::default();
        // SAFETY: The live handle, borrowed ID and output storage remain valid during the call.
        status(unsafe {
            call("dmd_catalogue_text", |error| {
                sys::dmd_catalogue_text(self.handle.as_ptr(), kind, bytes(id), &mut out, error)
            })
        }?)?;
        // SAFETY: The returned span is copied before any subsequent handle call.
        unsafe { copy_text(out) }
    }

    pub fn diagnostic(&mut self, index: u64) -> Result<Option<Diagnostic>, Error> {
        let mut out = sys::DmdDiagnostic::default();
        // SAFETY: The live handle is exclusively borrowed and output storage is valid.
        let code = unsafe {
            call("dmd_catalogue_diagnostic", |error| {
                sys::dmd_catalogue_diagnostic(self.handle.as_ptr(), index, &mut out, error)
            })
        }?;
        if code == sys::DMD_END {
            return Ok(None);
        }
        status(code)?;
        // SAFETY: Both returned spans remain live until the next handle call.
        unsafe {
            Ok(Some(Diagnostic {
                kind: out.kind,
                name: copy_text(out.name)?,
                message: copy_text(out.message)?,
            }))
        }
    }

    pub fn create_run(
        &mut self,
        id: &str,
        parameters: &[(&str, &str)],
        stream: Stream,
    ) -> Result<Run, Error> {
        let parameters: Vec<_> = parameters
            .iter()
            .map(|(name, value)| sys::DmdParameter {
                name: bytes(name),
                value: bytes(value),
            })
            .collect();
        let mut out = std::ptr::null_mut();
        // SAFETY: The live catalogue and borrowed inputs remain valid for the call; the ABI copies them.
        status(unsafe {
            call("dmd_run_create", |error| {
                sys::dmd_run_create(
                    self.handle.as_ptr(),
                    bytes(id),
                    parameters.as_ptr(),
                    parameters.len() as u64,
                    &stream.0,
                    &mut out,
                    error,
                )
            })
        }?)?;
        Ok(Run {
            handle: NonNull::new(out).ok_or(Error::Contract("null run"))?,
            thread: PhantomData,
            stream,
            finalized: false,
        })
    }
}

impl Drop for Catalogue {
    fn drop(&mut self) {
        // SAFETY: This owner closes its live handle exactly once on its creating thread.
        unsafe { sys::dmd_catalogue_close(self.handle.as_ptr()) };
    }
}

pub struct Run {
    handle: NonNull<sys::DmdRun>,
    thread: PhantomData<Rc<()>>,
    stream: Stream,
    finalized: bool,
}
#[derive(Debug)]
pub struct ResultData {
    pub identifier: String,
    pub payload: Vec<u8>,
}

impl Run {
    pub fn submit(&mut self, block: &Block) -> Result<(), Error> {
        if self.finalized {
            return Err(Error::Input("run is finalized"));
        }
        if block.samples.len() != self.stream.scalars(block.frames)? {
            return Err(Error::Input("sample count does not match frames"));
        }
        let raw = sys::DmdBlock {
            samples: block.samples.as_ptr(),
            frames: block.frames,
            source_position: block.source_position,
        };
        // SAFETY: The live unfinalized run receives a validated buffer, borrowed synchronously.
        status(unsafe {
            call("dmd_run_submit", |error| {
                sys::dmd_run_submit(self.handle.as_ptr(), &raw, error)
            })
        }?)
    }

    pub fn next_result(&mut self) -> Result<Option<ResultData>, Error> {
        let mut out = sys::DmdResult::default();
        // SAFETY: The live handle is exclusively borrowed and output storage is valid.
        let code = unsafe {
            call("dmd_run_next", |error| {
                sys::dmd_run_next(self.handle.as_ptr(), &mut out, error)
            })
        }?;
        if code == sys::DMD_END {
            return Ok(None);
        }
        status(code)?;
        // SAFETY: Both spans come from the successful call and are copied before another call.
        unsafe {
            Ok(Some(ResultData {
                identifier: copy_text(out.identifier)?,
                payload: copy_bytes(out.payload)?,
            }))
        }
    }

    pub fn finalize(&mut self) -> Result<(), Error> {
        if self.finalized {
            return Err(Error::Input("run is already finalized"));
        }
        self.finalized = true;
        // SAFETY: The live run has not been finalized previously.
        status(unsafe {
            call("dmd_run_finalize", |error| {
                sys::dmd_run_finalize(self.handle.as_ptr(), error)
            })
        }?)
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        // SAFETY: This owner destroys its live handle exactly once on its creating thread.
        unsafe { sys::dmd_run_destroy(self.handle.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_diagnostic_is_owned_before_the_next_call() {
        let mut borrowed = String::from("Cannot decode signal");
        // SAFETY: The fixture supplies a valid diagnostic span until call returns.
        let failure = unsafe {
            call("dmd_signal_open", |error| {
                assert_eq!(error.code, 0);
                assert!(error.message.data.is_null());
                assert_eq!(error.message.len, 0);
                error.code = 42;
                error.message = bytes(&borrowed);
                sys::DMD_ERROR
            })
        }
        .unwrap_err();
        borrowed.clear();
        let Error::Core {
            operation,
            code,
            message,
        } = &failure
        else {
            panic!("expected core diagnostic, got {failure}");
        };
        assert_eq!(*operation, "dmd_signal_open");
        assert_eq!(*code, 42);
        assert_eq!(message, "Cannot decode signal");
        assert_eq!(
            failure.to_string(),
            "native operation dmd_signal_open failed with diagnostic 42: Cannot decode signal"
        );
    }

    #[test]
    fn empty_diagnostics_and_iteration_statuses_are_preserved() {
        // SAFETY: A zero-length null span is permitted by the ABI.
        let failure = unsafe {
            call("dmd_run_next", |error| {
                error.code = 7;
                sys::DMD_ERROR
            })
        }
        .unwrap_err();
        assert!(
            matches!(failure, Error::Core { operation: "dmd_run_next", code: 7, message } if message.is_empty())
        );
        for code in [sys::DMD_OK, sys::DMD_END] {
            // SAFETY: These statuses have no diagnostic span to retain.
            assert_eq!(unsafe { call("dmd_run_next", |_| code) }.unwrap(), code);
        }
    }
}
