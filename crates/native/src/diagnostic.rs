use crate::Error;

pub(crate) enum Status {
    Returned(i32),
    Failed(i32),
}

pub(crate) fn outcome(
    operation: &'static str,
    status: Status,
    message: Option<String>,
) -> Result<i32, Error> {
    match status {
        Status::Returned(code) => Ok(code),
        Status::Failed(code) => Err(Error::Core {
            operation,
            code,
            message: message.unwrap_or_default(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_diagnostic_retains_the_owned_message() {
        let mut borrowed = String::from("Cannot decode signal");
        let failure = outcome(
            "dmd_signal_open",
            Status::Failed(42),
            Some(borrowed.clone()),
        )
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
    fn empty_diagnostics_are_preserved() {
        for message in [None, Some(String::new())] {
            let failure = outcome("dmd_run_next", Status::Failed(7), message).unwrap_err();
            assert!(
                matches!(failure, Error::Core { operation: "dmd_run_next", code: 7, message } if message.is_empty())
            );
        }
    }

    #[test]
    fn non_error_statuses_are_preserved() {
        for code in [0, 1, 17] {
            assert_eq!(
                outcome("dmd_run_next", Status::Returned(code), None).unwrap(),
                code
            );
        }
    }
}
