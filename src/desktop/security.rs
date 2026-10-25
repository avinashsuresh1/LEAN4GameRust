use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SecurityError {
    InvalidUrl(String),
    DisallowedScheme(String),
    DisallowedHost(String),
    PathTraversalAttempt(String),
    PayloadTooLarge(String),
    InvalidKey(String),
    InvalidValue(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InvalidUrl(msg) => write!(f, "Security error: invalid URL: {}", msg),
            SecurityError::DisallowedScheme(scheme) => {
                write!(f, "Security error: disallowed URI scheme '{}'", scheme)
            }
            SecurityError::DisallowedHost(host) => {
                write!(f, "Security error: untrusted host '{}' is blocked", host)
            }
            SecurityError::PathTraversalAttempt(path) => {
                write!(f, "Security error: path traversal attempt blocked: '{}'", path)
            }
            SecurityError::PayloadTooLarge(msg) => {
                write!(f, "Security error: payload too large: {}", msg)
            }
            SecurityError::InvalidKey(msg) => write!(f, "Security error: invalid key: {}", msg),
            SecurityError::InvalidValue(msg) => write!(f, "Security error: invalid value: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Central security policy enforcer for the desktop application.
/// Protects against malicious navigation, URI scheme exploits, path traversal,
/// and memory exhaustion attacks.
pub struct SecurityPolicy;

impl SecurityPolicy {
    pub const MAX_KEYS: usize = 10_000;
    pub const MAX_KEY_LENGTH: usize = 512;
    pub const MAX_VALUE_LENGTH: usize = 2 * 1024 * 1024; // 2 MB per item
    pub const MAX_TOTAL_PAYLOAD_BYTES: usize = 25 * 1024 * 1024; // 25 MB total

    /// Checks if a navigation URL is permitted to load inside the desktop webview.
    /// Only the official Lean 4 Game server, GitHub community game subpaths, and
    /// local loopback development servers (127.0.0.1, localhost) are permitted.
    pub fn is_url_allowed(url_str: &str) -> bool {
        let parsed = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let scheme = parsed.scheme().to_lowercase();
        let host = match parsed.host_str() {
            Some(h) => h.to_lowercase(),
            None => return false,
        };

        // Strictly allow only http and https. Dangerous schemes like file:, javascript:, data: are blocked.
        if scheme != "https" && scheme != "http" {
            return false;
        }

        // Local loopback is permitted on both http and https for self-contained local servers
        if host == "127.0.0.1" || host == "localhost" || host == "::1" || host == "[::1]" {
            return true;
        }

        // External connections MUST be https and strictly belong to approved origins
        if scheme == "https" {
            if host == "adam.math.hhu.de" || host.ends_with(".adam.math.hhu.de") {
                return true;
            }
            if host == "leanprover-community.github.io" {
                return true;
            }
        }

        false
    }

    /// Verifies that a target file path does not escape the designated base directory.
    /// Prevents directory traversal attacks such as `../../etc/passwd` or `..\..\..\Windows`.
    pub fn validate_save_path(base_dir: &Path, requested_path: &Path) -> Result<PathBuf, SecurityError> {
        for component in requested_path.components() {
            if let Component::ParentDir = component {
                return Err(SecurityError::PathTraversalAttempt(
                    requested_path.to_string_lossy().to_string(),
                ));
            }
        }

        let combined = if requested_path.is_relative() {
            base_dir.join(requested_path)
        } else {
            requested_path.to_path_buf()
        };

        for component in combined.components() {
            if let Component::ParentDir = component {
                return Err(SecurityError::PathTraversalAttempt(
                    combined.to_string_lossy().to_string(),
                ));
            }
        }

        Ok(combined)
    }

    /// Validates an IPC save payload to prevent heap exhaustion or disk corruption.
    pub fn validate_ipc_payload(data: &HashMap<String, String>) -> Result<(), SecurityError> {
        if data.len() > Self::MAX_KEYS {
            return Err(SecurityError::PayloadTooLarge(format!(
                "Too many keys: {} (max allowed is {})",
                data.len(),
                Self::MAX_KEYS
            )));
        }

        let mut total_bytes = 0;
        for (k, v) in data {
            if k.len() > Self::MAX_KEY_LENGTH {
                return Err(SecurityError::InvalidKey(format!(
                    "Key length {} exceeds limit of {}",
                    k.len(),
                    Self::MAX_KEY_LENGTH
                )));
            }
            if k.contains('\0') {
                return Err(SecurityError::InvalidKey("Key contains null byte".to_string()));
            }

            if v.len() > Self::MAX_VALUE_LENGTH {
                return Err(SecurityError::InvalidValue(format!(
                    "Value for key '{}' ({} bytes) exceeds limit of {} bytes",
                    k,
                    v.len(),
                    Self::MAX_VALUE_LENGTH
                )));
            }

            total_bytes += k.len() + v.len();
            if total_bytes > Self::MAX_TOTAL_PAYLOAD_BYTES {
                return Err(SecurityError::PayloadTooLarge(format!(
                    "Total payload size exceeds limit of {} bytes",
                    Self::MAX_TOTAL_PAYLOAD_BYTES
                )));
            }
        }

        Ok(())
    }
}
