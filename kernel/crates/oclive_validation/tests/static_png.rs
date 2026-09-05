#![cfg(feature = "media-png")]

use oclive_validation::static_png::{
    validate_static_png, StaticPngError, StaticPngInfo, StaticPngInvalid, StaticPngLimits,
    StaticPngUnsupported,
};

fn limits() -> StaticPngLimits {
    StaticPngLimits {
        max_input_bytes: 1024 * 1024,
        max_width: 256,
        max_height: 256,
        max_pixels: 256 * 256,
        max_output_bytes: 1024 * 1024,
        max_decoder_bytes: 1024 * 1024,
    }
}

fn rgba(width: u32, height: u32, animated: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    if animated {
        encoder.set_animated(1, 0).unwrap();
    }
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(&vec![127; width as usize * height as usize * 4])
        .unwrap();
    writer.finish().unwrap();
    bytes
}

fn invalid(reason: StaticPngInvalid) -> Result<StaticPngInfo, StaticPngError> {
    Err(StaticPngError::Invalid(reason))
}

// Test-only framing/CRC helpers let corrupt compressed bytes retain a valid outer
// CRC, and let unknown/unsupported chunks be inserted without new dependencies.
fn crc(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            value = (value >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(value & 1));
        }
    }
    !value
}

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut bytes = (data.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(kind);
    bytes.extend_from_slice(data);
    bytes.extend_from_slice(&crc(&bytes[4..]).to_be_bytes());
    bytes
}

fn replace_chunk(bytes: &mut [u8], wanted: &[u8; 4], edit: impl FnOnce(&mut [u8])) {
    let mut offset = 8;
    loop {
        let len = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        if &bytes[offset + 4..offset + 8] == wanted {
            edit(&mut bytes[offset + 8..offset + 8 + len]);
            let checksum = crc(&bytes[offset + 4..offset + 8 + len]).to_be_bytes();
            bytes[offset + 8 + len..offset + 12 + len].copy_from_slice(&checksum);
            return;
        }
        offset += len + 12;
        assert!(offset < bytes.len());
    }
}

#[test]
fn decodes_static_rgba_and_accepts_exact_input_dimension_pixel_and_output_limits() {
    let bytes = rgba(3, 2, false);
    let limits = StaticPngLimits {
        max_input_bytes: bytes.len(),
        max_width: 3,
        max_height: 2,
        max_pixels: 6,
        max_output_bytes: 24,
        ..limits()
    };
    assert_eq!(
        validate_static_png(&bytes, &limits).unwrap(),
        StaticPngInfo {
            width: 3,
            height: 2,
            decoded_bytes: 24,
        }
    );
}

#[test]
fn accepts_grayscale_16bit_and_indexed_static_png_without_rgba8_assumptions() {
    for (color, depth, data, palette) in [
        (png::ColorType::Grayscale, png::BitDepth::One, vec![0], None),
        (
            png::ColorType::Rgba,
            png::BitDepth::Sixteen,
            vec![0; 8],
            None,
        ),
        (
            png::ColorType::Indexed,
            png::BitDepth::Eight,
            vec![0],
            Some(vec![255, 0, 0]),
        ),
    ] {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
        encoder.set_color(color);
        encoder.set_depth(depth);
        if let Some(palette) = palette {
            encoder.set_palette(palette);
        }
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&data).unwrap();
        writer.finish().unwrap();
        assert_eq!(
            validate_static_png(&bytes, &limits())
                .unwrap()
                .decoded_bytes,
            data.len()
        );
    }
}

#[test]
fn rejects_each_budget_overrun_independently() {
    let bytes = rgba(3, 2, false);
    for (policy, reason) in [
        (
            StaticPngLimits {
                max_input_bytes: bytes.len() - 1,
                ..limits()
            },
            StaticPngInvalid::InputLimit,
        ),
        (
            StaticPngLimits {
                max_width: 2,
                ..limits()
            },
            StaticPngInvalid::DimensionLimit,
        ),
        (
            StaticPngLimits {
                max_height: 1,
                ..limits()
            },
            StaticPngInvalid::DimensionLimit,
        ),
        (
            StaticPngLimits {
                max_pixels: 5,
                ..limits()
            },
            StaticPngInvalid::PixelLimit,
        ),
        (
            StaticPngLimits {
                max_output_bytes: 23,
                ..limits()
            },
            StaticPngInvalid::OutputLimit,
        ),
        (
            StaticPngLimits {
                max_decoder_bytes: 1,
                ..limits()
            },
            StaticPngInvalid::DecoderLimit,
        ),
    ] {
        assert_eq!(validate_static_png(&bytes, &policy), invalid(reason));
    }
}

#[test]
fn zero_budgets_are_caller_errors_not_media_invalidity() {
    for policy in [
        StaticPngLimits {
            max_input_bytes: 0,
            ..limits()
        },
        StaticPngLimits {
            max_width: 0,
            ..limits()
        },
        StaticPngLimits {
            max_height: 0,
            ..limits()
        },
        StaticPngLimits {
            max_pixels: 0,
            ..limits()
        },
        StaticPngLimits {
            max_output_bytes: 0,
            ..limits()
        },
        StaticPngLimits {
            max_decoder_bytes: 0,
            ..limits()
        },
    ] {
        assert_eq!(
            validate_static_png(b"", &policy),
            Err(StaticPngError::InvalidLimits)
        );
    }
}

#[test]
fn non_png_is_unsupported_even_if_larger_than_png_input_budget() {
    // This adapter does not claim any of these are valid media.
    for bytes in [
        b"\xff\xd8\xff\xe0JPEG".as_slice(),
        b"RIFF1234WEBPpayload",
        b"GIF89a",
        b"<svg></svg>",
        b"unknown",
        b"",
    ] {
        assert_eq!(
            validate_static_png(
                bytes,
                &StaticPngLimits {
                    max_input_bytes: 1,
                    ..limits()
                }
            ),
            Err(StaticPngError::Unsupported(StaticPngUnsupported::NotPng))
        );
    }
}

#[test]
fn rejects_every_nonempty_truncation_of_valid_png() {
    let bytes = rgba(1, 1, false);
    for end in 1..bytes.len() {
        assert_eq!(
            validate_static_png(&bytes[..end], &limits()),
            invalid(StaticPngInvalid::Malformed),
            "cut at {end}"
        );
    }
}

#[test]
fn rejects_header_and_payload_crc_corruption() {
    let original = rgba(1, 1, false);
    for offset in [29, 41] {
        let mut bytes = original.clone();
        bytes[offset] ^= 1;
        assert_eq!(
            validate_static_png(&bytes, &limits()),
            invalid(StaticPngInvalid::Malformed)
        );
    }
}

#[test]
fn rejects_bad_adler_and_bad_compressed_stream_even_with_valid_chunk_crc() {
    for bad_adler in [true, false] {
        let mut bytes = rgba(1, 1, false);
        replace_chunk(&mut bytes, b"IDAT", |data| {
            if bad_adler {
                *data.last_mut().unwrap() ^= 1;
            } else {
                data[0] = 0;
            }
        });
        assert_eq!(
            validate_static_png(&bytes, &limits()),
            invalid(StaticPngInvalid::Malformed)
        );
    }
}

#[test]
fn rejects_empty_image_dimensions_and_stops_large_dimensions_before_pixel_decode() {
    for width in [0_u32, 100_000] {
        let mut bytes = rgba(1, 1, false);
        replace_chunk(&mut bytes, b"IHDR", |data| {
            data[..4].copy_from_slice(&width.to_be_bytes())
        });
        let reason = if width == 0 {
            StaticPngInvalid::Malformed
        } else {
            StaticPngInvalid::DimensionLimit
        };
        assert_eq!(validate_static_png(&bytes, &limits()), invalid(reason));
    }
}

#[test]
fn frame_decode_is_required_not_just_valid_headers_and_crc() {
    let mut bytes = rgba(1, 1, false);
    replace_chunk(&mut bytes, b"IHDR", |data| {
        data[4..8].copy_from_slice(&2_u32.to_be_bytes())
    });
    assert_eq!(
        validate_static_png(&bytes, &limits()),
        invalid(StaticPngInvalid::Malformed)
    );
}

#[test]
fn apng_is_unsupported_not_a_successful_static_first_frame() {
    assert_eq!(
        validate_static_png(&rgba(1, 1, true), &limits()),
        Err(StaticPngError::Unsupported(StaticPngUnsupported::Animation))
    );
    // Even malformed/late animation metadata must not silently fall back to a
    // valid static result. Unsupported does not validate the animation itself.
    for kind in [b"acTL", b"fcTL", b"fdAT"] {
        let mut bytes = rgba(1, 1, false);
        let end = bytes.len() - 12;
        bytes.splice(end..end, chunk(kind, &[]));
        assert_eq!(
            validate_static_png(&bytes, &limits()),
            Err(StaticPngError::Unsupported(StaticPngUnsupported::Animation))
        );
    }
}

#[test]
fn rejects_trailing_bytes_duplicate_images_and_overflowing_chunk_lengths() {
    let original = rgba(1, 1, false);
    for suffix in [b"junk".as_slice(), original.as_slice()] {
        let mut bytes = original.clone();
        bytes.extend_from_slice(suffix);
        assert_eq!(
            validate_static_png(&bytes, &limits()),
            invalid(StaticPngInvalid::Malformed)
        );
    }
    let mut bytes = original;
    bytes[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(
        validate_static_png(&bytes, &limits()),
        invalid(StaticPngInvalid::Malformed)
    );
}

#[test]
fn consumes_and_checks_chunks_after_pixel_data() {
    let mut bytes = rgba(1, 1, false);
    let end = bytes.len() - 12;
    bytes.splice(end..end, chunk(b"tEXt", b"note\0private-metadata"));
    assert!(validate_static_png(&bytes, &limits()).is_ok());
    bytes[end + 8] ^= 1; // Keep text skipped, but fail its CRC.
    let error = validate_static_png(&bytes, &limits()).unwrap_err();
    assert_eq!(error, StaticPngError::Invalid(StaticPngInvalid::Malformed));
    assert!(!error.to_string().contains("private-metadata"));
}

#[test]
fn local_snapshots_and_logical_contract_remain_independent_of_media_support() {
    use oclive_validation::{
        load_minimal_role_local_assets, validate_minimal_role_definition, MinimalRoleDefinition,
    };
    let dir = tempfile::tempdir().unwrap();
    let png = rgba(1, 1, false);
    std::fs::write(dir.path().join("portrait"), &png).unwrap();
    std::fs::write(dir.path().join("other.webp"), b"RIFF1234WEBP").unwrap();
    let role = MinimalRoleDefinition {
        persona_prompt: "A guide.".into(),
        visual_assets: vec!["portrait".into(), "other.webp".into()],
    };
    let before = role.clone();
    let snapshots = load_minimal_role_local_assets(dir.path(), &role, 1024, 2048).unwrap();
    assert!(validate_static_png(&snapshots[0], &limits()).is_ok());
    assert!(matches!(
        validate_static_png(&snapshots[1], &limits()),
        Err(StaticPngError::Unsupported(_))
    ));
    validate_minimal_role_definition(&role).unwrap();
    assert_eq!(role, before);
    assert_eq!(snapshots[0], png);
}
