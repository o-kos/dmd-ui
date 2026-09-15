use dmd_core_sys as sys;

use crate::{
    Error,
    diagnostic::{self, Status},
};

// The callback must return an ABI status and diagnostic, keeping its span live
// until this returns. Each production callback makes exactly one core call.
pub(super) unsafe fn call(
    operation: &'static str,
    invoke: impl FnOnce(&mut sys::DmdError) -> i32,
) -> Result<i32, Error> {
    let mut error = sys::DmdError::default();
    let code = invoke(&mut error);
    let (status, message) = if code == sys::DMD_ERROR {
        // SAFETY: The caller guarantees the diagnostic remains live. Copy before another
        // core call; the ABI owns this span and forbids callers from freeing it.
        let message = unsafe { copy_text(error.message) }?;
        (Status::Failed(error.code), Some(message))
    } else {
        (Status::Returned(code), None)
    };
    diagnostic::outcome(operation, status, message)
}

pub(super) fn bytes(value: &str) -> sys::DmdBytes {
    sys::DmdBytes {
        data: value.as_ptr(),
        len: value.len() as u64,
    }
}

// SAFETY requirements are discharged at each call: the core returned this span
// from an operation and no subsequent operation has invalidated it.
pub(super) unsafe fn copy_bytes(value: sys::DmdBytes) -> Result<Vec<u8>, Error> {
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

pub(super) unsafe fn copy_text(value: sys::DmdBytes) -> Result<String, Error> {
    // SAFETY: The caller provides the same live returned span required by copy_bytes.
    String::from_utf8(unsafe { copy_bytes(value) }?).map_err(|_| Error::Contract("invalid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_text_retains_utf8_after_source_is_dropped() {
        let source = String::from("Invalid frequency: 10 µHz");
        // SAFETY: The fixture owns the UTF-8 span and keeps it live during the copy.
        let copied = unsafe { copy_text(bytes(&source)) }.unwrap();
        drop(source);
        assert_eq!(copied, "Invalid frequency: 10 µHz");
    }

    #[test]
    fn call_initializes_and_copies_the_ffi_diagnostic() {
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
        assert!(
            matches!(failure, Error::Core { message, .. } if message == "Cannot decode signal")
        );
    }

    #[test]
    fn call_accepts_empty_ffi_diagnostics_and_iteration_statuses() {
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

    #[test]
    fn call_retains_message_after_next_call_overwrites_core_buffer() {
        let mut buffer = b"Cannot decode signal".to_vec();
        // SAFETY: The simulated core owns this valid UTF-8 span and keeps it
        // immutable until the next call, after the adapter has returned.
        let failure = unsafe {
            call("dmd_run_next", |error| {
                error.code = 42;
                error.message = sys::DmdBytes {
                    data: buffer.as_ptr(),
                    len: buffer.len() as u64,
                };
                sys::DMD_ERROR
            })
        }
        .unwrap_err();
        // SAFETY: The next simulated call may reuse its own returned storage.
        // DMD_END supplies no diagnostic span; the adapter never frees the buffer.
        let next = unsafe {
            call("dmd_run_next", |_| {
                buffer.fill(b'x');
                sys::DMD_END
            })
        }
        .unwrap();
        assert_eq!(next, sys::DMD_END);
        assert_eq!(buffer, vec![b'x'; "Cannot decode signal".len()]);
        assert!(
            matches!(failure, Error::Core { operation: "dmd_run_next", code: 42, message }
                if message == "Cannot decode signal")
        );
    }
}
