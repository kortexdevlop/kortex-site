use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::ImageEncoder;
use serde_json::json;
use tool_compresseur_images::{FILE_LIMIT, compress_image};

fn png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let pixels = vec![color; (width * height) as usize];
    let raw: Vec<u8> = pixels.into_iter().flatten().collect();
    let mut encoded = Vec::new();
    image::codecs::png::PngEncoder::new(&mut encoded)
        .write_image(&raw, width, height, image::ExtendedColorType::Rgba8)
        .unwrap();
    encoded
}

fn opaque_png(width: u32, height: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width * height * 3) as usize);
    let mut state = 0x1234_5678u32;
    for _ in 0..width * height * 3 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        raw.push((state >> 24) as u8);
    }
    let mut encoded = Vec::new();
    image::codecs::png::PngEncoder::new(&mut encoded)
        .write_image(&raw, width, height, image::ExtendedColorType::Rgb8)
        .unwrap();
    encoded
}

#[test]
fn compresses_opaque_images_and_preserves_transparency() {
    let source = opaque_png(80, 60);
    let output = compress_image(&json!({
        "data_base64":STANDARD.encode(source),
        "quality":60
    }))
    .unwrap();
    assert_eq!(output["format"], "jpeg");
    assert_eq!(output["width"], 80);
    assert_eq!(output["height"], 60);
    assert_eq!(output["reduced"], true);
    assert!(
        output["compressed_size"].as_u64().unwrap() <= output["original_size"].as_u64().unwrap()
    );

    let transparent = png(2, 1, [10, 20, 30, 0]);
    let output = compress_image(&json!({
        "data_base64":STANDARD.encode(transparent),
        "quality":60
    }))
    .unwrap();
    assert_eq!(output["format"], "png");
    let bytes = STANDARD
        .decode(output["data_base64"].as_str().unwrap())
        .unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!(decoded.get_pixel(0, 0).0[3], 0);
}

#[test]
fn accepts_webp_input() {
    let raw: Vec<u8> = vec![[120, 80, 40]; 80 * 60].into_iter().flatten().collect();
    let mut source = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut source)
        .write_image(&raw, 80, 60, image::ExtendedColorType::Rgb8)
        .unwrap();
    let output = compress_image(&json!({
        "data_base64":STANDARD.encode(source),
        "quality":75
    }))
    .unwrap();
    assert_eq!(output["width"], 80);
    assert_eq!(output["height"], 60);
    assert!(["jpeg", "webp"].contains(&output["format"].as_str().unwrap()));
}

#[test]
fn rejects_invalid_formats_dimensions_and_quality() {
    assert_eq!(
        compress_image(&json!({"data_base64":"bm90IGFuIGltYWdl","quality":75})).unwrap_err(),
        "UNSUPPORTED_FORMAT"
    );
    assert_eq!(
        compress_image(
            &json!({"data_base64":STANDARD.encode(png(2001, 2000, [0,0,0,255])),"quality":75})
        )
        .unwrap_err(),
        "DIMENSIONS_LIMIT"
    );
    assert_eq!(
        compress_image(&json!({"data_base64":STANDARD.encode(png(1,1,[0,0,0,255])),"quality":10}))
            .unwrap_err(),
        "INVALID_QUALITY"
    );
    assert_eq!(
        compress_image(
            &json!({"data_base64":STANDARD.encode(vec![0; FILE_LIMIT as usize + 1]),"quality":75})
        )
        .unwrap_err(),
        "TOO_LARGE"
    );
}
