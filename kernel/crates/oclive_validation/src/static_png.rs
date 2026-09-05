//! Optional static-PNG media capability, enabled only by `media-png`.
//!
//! This module validates a byte snapshot, not a minimal role definition. PNG is
//! not a required role format. Unsupported media must not be treated as an invalid
//! role. There is no filesystem/network access, rendering, or lifecycle wiring.

use std::{fmt, io::Cursor};

const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// Caller-owned budgets, never authored role fields. Every limit must be positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticPngLimits {
    pub max_input_bytes: usize,
    pub max_width: u32,
    pub max_height: u32,
    pub max_pixels: u64,
    /// Maximum identity-transformed decoded frame buffer, independently bounded.
    pub max_output_bytes: usize,
    /// Passed to `png::Limits`: best-effort decoder allocations, excluding the
    /// input snapshot and our output buffer. Not a hard total-memory limit.
    pub max_decoder_bytes: usize,
}

/// Successful pixel decode under the supplied limits, not a rendering guarantee.
/// No decoded pixels or optional metadata are returned or cached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticPngInfo {
    pub width: u32,
    pub height: u32,
    /// Size of the identity-transformed frame, not necessarily RGBA8.
    pub decoded_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticPngUnsupported {
    /// No PNG signature. Includes other formats and unidentifiable bytes; their
    /// validity has not been assessed by this adapter.
    NotPng,
    /// An APNG chunk marker is present. Animation validity is not assessed.
    Animation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticPngInvalid {
    Malformed,
    InputLimit,
    DimensionLimit,
    PixelLimit,
    OutputLimit,
    DecoderLimit,
}

/// Distinguishes unsupported capability, media rejection under a caller policy,
/// and adapter/caller failures. None of these changes the logical role contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticPngError {
    Unsupported(StaticPngUnsupported),
    Invalid(StaticPngInvalid),
    InvalidLimits,
    AllocationFailed,
    DecoderFailure,
}

impl fmt::Display for StaticPngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Only fixed enum names: never echo input bytes, paths or decoder details.
        write!(f, "static PNG capability: {self:?}")
    }
}

impl std::error::Error for StaticPngError {}

/// Validate static PNG pixels using explicit caller budgets and checksum checks.
///
/// Enable `oclive_validation`'s opt-in `media-png` feature to use this function.
/// A non-PNG signature returns `Unsupported(NotPng)` before input-size checks;
/// a nonempty truncated PNG-signature prefix is instead malformed. APNG markers
/// are unsupported, never silently validated as a static first frame. Unsupported
/// does not prove media is valid and must not invalidate a minimal role by itself.
///
/// Dimensions/pixels are checked after IHDR, before the remaining metadata/frame
/// is decoded. Output and decoder allocation budgets are separate. The PNG crate's
/// allocation accounting is best-effort; allocator overhead and some internal
/// allocations are not covered. No hard memory limit, timeout or process isolation
/// is provided. Use host-appropriate limits, not untrusted pack-supplied budgets.
///
/// Pixel data is decoded in identity mode and discarded, with no color conversion,
/// EXIF interpretation, re-encoding or thumbnails. Text and ICC chunks are skipped.
/// Optional metadata semantics are not validated. The full static frame and IEND
/// must be consumed; trailing bytes are rejected by this adapter's framing policy.
/// Validation is not a sanitizer: do not assume another decoder has identical
/// behavior or protections when it later consumes the same snapshot.
///
/// # Errors
///
/// Returns typed capability/media errors, invalid caller limits, allocation
/// failure, or unexpected decoder API failure. No raw decoder error is exposed.
///
/// ```
/// use oclive_validation::static_png::{
///     validate_static_png, StaticPngError, StaticPngLimits, StaticPngUnsupported,
/// };
/// let limits = StaticPngLimits {
///     max_input_bytes: 4 * 1024 * 1024,
///     max_width: 2048, max_height: 2048, max_pixels: 2048 * 2048,
///     max_output_bytes: 32 * 1024 * 1024, max_decoder_bytes: 16 * 1024 * 1024,
/// };
/// assert_eq!(validate_static_png(b"not a PNG", &limits),
///     Err(StaticPngError::Unsupported(StaticPngUnsupported::NotPng)));
/// ```
pub fn validate_static_png(
    bytes: &[u8],
    limits: &StaticPngLimits,
) -> Result<StaticPngInfo, StaticPngError> {
    if limits.max_input_bytes == 0
        || limits.max_width == 0
        || limits.max_height == 0
        || limits.max_pixels == 0
        || limits.max_output_bytes == 0
        || limits.max_decoder_bytes == 0
    {
        return Err(StaticPngError::InvalidLimits);
    }
    if !bytes.starts_with(SIGNATURE) {
        return Err(if !bytes.is_empty() && SIGNATURE.starts_with(bytes) {
            StaticPngError::Invalid(StaticPngInvalid::Malformed)
        } else {
            StaticPngError::Unsupported(StaticPngUnsupported::NotPng)
        });
    }
    if bytes.len() > limits.max_input_bytes {
        return Err(StaticPngError::Invalid(StaticPngInvalid::InputLimit));
    }
    check_static_framing(bytes)?;
    let mut options = png::DecodeOptions::default();
    options.set_ignore_checksums(false); // Includes Adler-32 (disabled by default).
    options.set_skip_ancillary_crc_failures(false);
    options.set_ignore_text_chunk(true);
    options.set_ignore_iccp_chunk(true);
    let mut decoder = png::Decoder::new_with_options(Cursor::new(bytes), options);
    decoder.set_limits(png::Limits {
        bytes: limits.max_decoder_bytes,
    });
    decoder.set_transformations(png::Transformations::IDENTITY);
    let header = decoder.read_header_info().map_err(decoder_error)?;
    let (width, height) = (header.width, header.height);
    if width > limits.max_width || height > limits.max_height {
        return Err(StaticPngError::Invalid(StaticPngInvalid::DimensionLimit));
    }
    if u64::from(width) * u64::from(height) > limits.max_pixels {
        return Err(StaticPngError::Invalid(StaticPngInvalid::PixelLimit));
    }
    let mut reader = decoder.read_info().map_err(decoder_error)?;
    let decoded_bytes = reader
        .output_buffer_size()
        .filter(|size| *size <= limits.max_output_bytes)
        .ok_or(StaticPngError::Invalid(StaticPngInvalid::OutputLimit))?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(decoded_bytes)
        .map_err(|_| StaticPngError::AllocationFailed)?;
    pixels.resize(decoded_bytes, 0);
    reader.next_frame(&mut pixels).map_err(decoder_error)?;
    reader.finish().map_err(decoder_error)?;
    Ok(StaticPngInfo {
        width,
        height,
        decoded_bytes,
    })
}

fn decoder_error(error: png::DecodingError) -> StaticPngError {
    match error {
        png::DecodingError::LimitsExceeded => {
            StaticPngError::Invalid(StaticPngInvalid::DecoderLimit)
        }
        png::DecodingError::IoError(_) | png::DecodingError::Format(_) => {
            StaticPngError::Invalid(StaticPngInvalid::Malformed)
        }
        png::DecodingError::Parameter(_) => StaticPngError::DecoderFailure,
    }
}

/// Only inspect bounded chunk envelopes and unsupported markers. CRC, IHDR and
/// pixel semantics remain owned by the PNG decoder, not a second PNG decoder.
/// This prevents permissive ancillary/APNG handling from accepting a static
/// fallback and makes the IEND/trailing-data policy explicit.
fn check_static_framing(bytes: &[u8]) -> Result<(), StaticPngError> {
    let malformed = StaticPngError::Invalid(StaticPngInvalid::Malformed);
    let mut remaining = &bytes[SIGNATURE.len()..];
    while remaining.len() >= 12 {
        let length = u32::from_be_bytes([remaining[0], remaining[1], remaining[2], remaining[3]]);
        let size = usize::try_from(length)
            .ok()
            .and_then(|length| length.checked_add(12))
            .filter(|size| *size <= remaining.len())
            .ok_or(malformed)?;
        let kind = &remaining[4..8];
        if matches!(kind, b"acTL" | b"fcTL" | b"fdAT") {
            return Err(StaticPngError::Unsupported(StaticPngUnsupported::Animation));
        }
        remaining = &remaining[size..];
        if kind == b"IEND" {
            return if length == 0 && remaining.is_empty() {
                Ok(())
            } else {
                Err(malformed)
            };
        }
    }
    Err(malformed)
}
