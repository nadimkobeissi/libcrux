#![doc = include_str!("KDF_Readme.md")]
#![allow(non_snake_case, non_camel_case_types)]

use hacspec_lib::*;

use super::errors::*;
use crate::{
    hkdf::{Algorithm, Error as CryptoError},
    specs,
};

type CryptoResult = Result<Bytes, CryptoError>;

/// ## Key Derivation Functions (KDFs)
///
/// | Value  | KDF         | Nh  | Reference |
/// | :----- | :---------- | --- | :-------- |
/// | 0x0000 | (reserved)  | N/A | N/A       |
/// | 0x0001 | HKDF-SHA256 | 32  | [RFC5869] |
/// | 0x0002 | HKDF-SHA384 | 48  | [RFC5869] |
/// | 0x0003 | HKDF-SHA512 | 64  | [RFC5869] |
///
/// ### KDF Identifiers
///
/// The "HPKE KDF Identifiers" registry lists identifiers for key derivation
/// functions defined for use with HPKE. These identifiers are two-byte values,
/// so the maximum possible value is 0xFFFF = 65535.
///
/// Template:
///
/// * Value: The two-byte identifier for the algorithm
/// * KDF: The name of the algorithm
/// * Nh: The output size of the Extract function in bytes
/// * Reference: Where this algorithm is defined
///
/// [RFC5869]: https://www.rfc-editor.org/info/rfc5869
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KDF {
    /// 0x0001
    HKDF_SHA256,
    /// 0x0002
    HKDF_SHA384,
    /// 0x0003
    HKDF_SHA512,
}

// pub type Error = u8;
// pub const UNKNOWN_ALGORITHM: Error = 1u8;
// pub const HKDF_INVALID_OUTPUT_LENGTH: Error = 2u8;
// pub const CRYPTO_ERROR: Error = 3u8;

pub type InputKeyMaterial = Bytes;
pub type Info = Bytes;

/// Get the numeric value of the `kdf_id`.
///
/// See [`KDF`] for details.
pub fn kdf_value(kdf_id: KDF) -> U16 {
    match kdf_id {
        KDF::HKDF_SHA256 => U16(0x0001u16),
        KDF::HKDF_SHA384 => U16(0x0002u16),
        KDF::HKDF_SHA512 => U16(0x0003u16),
    }
}

/// The output size of the `Extract()` function in bytes.
///
/// See [`KDF`] for details.
pub fn Nh(kdf_id: KDF) -> usize {
    match kdf_id {
        KDF::HKDF_SHA256 => 32,
        KDF::HKDF_SHA384 => 48,
        KDF::HKDF_SHA512 => 64,
    }
}

/// The string literal "HPKE-v1" used in [`LabeledExtract()`] and [`LabeledExpand()`]
/// ensures that any secrets derived in HPKE are bound to the scheme's name
/// and version, even when possibly derived from the same Diffie-Hellman or
/// KEM shared secret as in another scheme or version.
fn hpke_version_label() -> Bytes {
    create_bytes!(0x48u8, 0x50u8, 0x4bu8, 0x45u8, 0x2du8, 0x76u8, 0x31u8)
}

fn hkdf_algorithm(alg: KDF) -> Algorithm {
    match alg {
        KDF::HKDF_SHA256 => Algorithm::Sha256,
        KDF::HKDF_SHA384 => Algorithm::Sha384,
        KDF::HKDF_SHA512 => Algorithm::Sha512,
    }
}

/// LabeledExtract
///
/// ```text
/// def LabeledExtract(salt, label, ikm):
///   labeled_ikm = concat("HPKE-v1", suite_id, label, ikm)
///   return Extract(salt, labeled_ikm)
/// ```
pub fn LabeledExtract(
    alg: KDF,
    suite_id: Bytes,
    salt: &Bytes,
    label: Bytes,
    ikm: &InputKeyMaterial,
) -> HpkeBytesResult {
    Ok(specs::hkdf::extract(
        hkdf_algorithm(alg),
        salt,
        &hpke_version_label()
            .concat_owned(suite_id)
            .concat_owned(label)
            .concat(ikm),
    ))
}

/// KDF: Labeled Expand
///
/// ```text
/// def LabeledExpand(prk, label, info, L):
///   labeled_info = concat(I2OSP(L, 2), "HPKE-v1", suite_id,
///                         label, info)
///   return Expand(prk, labeled_info, L)
/// ```
pub fn LabeledExpand(
    alg: KDF,
    suite_id: Bytes,
    prk: &Bytes,
    label: Bytes,
    info: Info,
    L: usize,
) -> HpkeBytesResult {
    if L > (255 * Nh(alg)) {
        // This check is mentioned explicitly in the spec because because it
        // must be adhered to when exporting secrets.
        // The check comes from HKDF and will be performed there again.
        HpkeBytesResult::Err(HpkeError::InvalidParameters)
    } else {
        match specs::hkdf::expand(
            hkdf_algorithm(alg),
            prk,
            &U16(L as u16)
                .into_bytes()
                .concat_owned(hpke_version_label())
                .concat_owned(suite_id)
                .concat_owned(label)
                .concat_owned(info),
            L,
        ) {
            CryptoResult::Ok(r) => HpkeBytesResult::Ok(r),
            CryptoResult::Err(_) => HpkeBytesResult::Err(HpkeError::CryptoError),
        }
    }
}
