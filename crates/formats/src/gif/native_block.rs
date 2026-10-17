// SPDX-License-Identifier: GPL-3.0-or-later

//! FractalForge's own GIF application-extension block.
//! See `docs/specs/gif-native-block.md`.

/// Application identifier (8 bytes). Part of the file format; never change it.
pub const APP_IDENTIFIER: &[u8; 8] = b"FRCTFRGE";

/// Application authentication code (3 bytes). Part of the file format.
pub const AUTH_CODE: &[u8; 3] = b"1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_is_ascii_and_fixed() {
        assert_eq!(APP_IDENTIFIER, b"FRCTFRGE");
        assert_eq!(AUTH_CODE, b"1.0");
        assert!(APP_IDENTIFIER.is_ascii());
    }
}
