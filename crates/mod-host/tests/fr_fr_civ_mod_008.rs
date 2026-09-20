//! Tests for FR-CIV-MOD-008 — HostState and host constants
//!
//! Epic: FR-CIV-MOD
//! Verifies HostState defaults and host API version constant.

#[cfg(test)]
mod fr_fr_civ_mod_008 {
    /// FR-CIV-MOD-008: HostState defaults without panic.
    #[test]
    fn host_state_default_no_panic() {
        let _state = civ_mod_host::HostState::default();
    }

    /// FR-CIV-MOD-008: HOST_CAPABILITY_API_VERSION is defined and non-empty.
    #[test]
    fn api_version_is_defined() {
        assert!(civ_mod_host::HOST_CAPABILITY_API_VERSION > 0);
    }

    /// FR-CIV-MOD-008: HOST_IMPORT_MODULE is non-empty.
    #[test]
    fn host_import_module_nonempty() {
        assert!(!civ_mod_host::HOST_IMPORT_MODULE.is_empty());
    }
}
