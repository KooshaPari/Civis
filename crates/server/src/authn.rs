//! Pure authentication primitives for server boundaries.
//!
//! This module only parses an HTTP `Authorization: Bearer <token>` value. It
//! deliberately does not verify, persist, or log credentials; transport and
//! token-verification integrations can build on this narrow contract later.

use std::fmt;

/// The only credential scheme currently accepted by the server boundary.
pub const BEARER_SCHEME: &str = "Bearer";

/// Failures produced while parsing an authorization header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthnError {
    /// The request did not include an authorization header.
    MissingAuthorization,
    /// The header did not contain exactly a scheme and credential.
    InvalidAuthorization,
    /// The header used a scheme other than [`BEARER_SCHEME`].
    UnsupportedScheme,
    /// The bearer credential was empty.
    EmptyToken,
    /// The bearer credential contained whitespace or control characters.
    InvalidToken,
}

impl fmt::Display for AuthnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingAuthorization => "authorization header is required",
            Self::InvalidAuthorization => "authorization header is malformed",
            Self::UnsupportedScheme => "authorization scheme is unsupported",
            Self::EmptyToken => "bearer token is empty",
            Self::InvalidToken => "bearer token is invalid",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AuthnError {}

/// An opaque bearer credential.
///
/// `Debug` intentionally redacts the credential so accidental structured
/// logging cannot disclose the token.
#[derive(Clone, PartialEq, Eq)]
pub struct BearerToken(String);

impl BearerToken {
    /// Parse a bearer authorization header into an opaque credential.
    pub fn parse(header: Option<&str>) -> Result<Self, AuthnError> {
        let header = header.ok_or(AuthnError::MissingAuthorization)?.trim();
        let mut fields = header.split_ascii_whitespace();
        let scheme = fields.next().ok_or(AuthnError::InvalidAuthorization)?;
        if !scheme.eq_ignore_ascii_case(BEARER_SCHEME) {
            return Err(AuthnError::UnsupportedScheme);
        }
        let token = fields.next().ok_or(AuthnError::InvalidAuthorization)?;
        if fields.next().is_some() {
            return Err(AuthnError::InvalidAuthorization);
        }
        if token.is_empty() {
            return Err(AuthnError::EmptyToken);
        }
        if token.chars().any(|character| character.is_ascii_control()) {
            return Err(AuthnError::InvalidToken);
        }
        Ok(Self(token.to_owned()))
    }

    /// Return the opaque token for a verifier; callers must not log it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for BearerToken {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for BearerToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerToken(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthnError, BearerToken};

    #[test]
    fn parses_bearer_scheme_case_insensitively() {
        let token = BearerToken::parse(Some("  bEaReR  opaque-token  ")).unwrap();
        assert_eq!(token.as_str(), "opaque-token");
    }

    #[test]
    fn rejects_missing_or_unsupported_authorization() {
        assert_eq!(
            BearerToken::parse(None),
            Err(AuthnError::MissingAuthorization)
        );
        assert_eq!(
            BearerToken::parse(Some("Basic abc")),
            Err(AuthnError::UnsupportedScheme)
        );
    }

    #[test]
    fn rejects_malformed_and_control_character_tokens() {
        assert_eq!(
            BearerToken::parse(Some("Bearer")),
            Err(AuthnError::InvalidAuthorization)
        );
        assert_eq!(
            BearerToken::parse(Some("Bearer one two")),
            Err(AuthnError::InvalidAuthorization)
        );
        assert_eq!(
            BearerToken::parse(Some("Bearer \u{0007}")),
            Err(AuthnError::InvalidToken)
        );
    }

    #[test]
    fn debug_redacts_credential() {
        let token = BearerToken::parse(Some("Bearer do-not-log")).unwrap();
        let debug = format!("{token:?}");
        assert_eq!(debug, "BearerToken(<redacted>)");
        assert!(!debug.contains("do-not-log"));
    }
}
