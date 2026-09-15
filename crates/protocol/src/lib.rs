//! Shared protocol boundary. Message schemas are intentionally deferred.

/// Event protocol version, independent of recording and C ABI versions.
pub const PROTOCOL_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    #[test]
    fn protocol_version_is_initial_version() {
        assert_eq!(super::PROTOCOL_VERSION, 1);
    }
}
