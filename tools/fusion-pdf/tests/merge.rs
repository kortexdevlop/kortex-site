use base64::{Engine as _, engine::general_purpose::STANDARD};
use miniz_oxide::deflate::compress_to_vec_zlib;
use serde_json::json;
use tool_fusion_pdf::{FILE_LIMIT, merge_pdfs, pdf};

/// Classic PDF: xref table, one page tree with an inherited MediaBox and Resources.
fn classic(texts: &[&str]) -> Vec<u8> {
    let mut out = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    let n = texts.len() as u32;
    let mut objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {n} /MediaBox [0 0 200 100] /Resources << /Font << /F1 3 0 R >> >> >>",
            (0..n)
                .map(|i| format!("{} 0 R", 4 + 2 * i))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
    ];
    for (i, text) in texts.iter().enumerate() {
        let content = 5 + 2 * i;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /Contents {content} 0 R >>"
        ));
        let stream = format!("BT /F1 12 Tf 10 50 Td ({text}) Tj ET");
        objects.push(format!(
            "<< /Length {} >>\nstream\n{stream}\nendstream",
            stream.len()
        ));
    }
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// Modern PDF: catalog, page tree and page live in a compressed object stream,
/// indexed by a compressed xref stream with a PNG "Up" predictor.
fn modern(text: &str) -> Vec<u8> {
    let members = [
        (1u32, "<< /Type /Catalog /Pages 2 0 R >>"),
        (
            2,
            "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 300 300] >>",
        ),
        (3, "<< /Type /Page /Parent 2 0 R /Contents 5 0 R >>"),
    ];
    let mut header = String::new();
    let mut body = String::new();
    for (id, text) in members {
        header.push_str(&format!("{id} {} ", body.len()));
        body.push_str(text);
        body.push(' ');
    }
    let objstm_data = compress_to_vec_zlib(format!("{header}{body}").as_bytes(), 6);
    let mut out = b"%PDF-1.5\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let objstm_at = out.len();
    out.extend_from_slice(
        format!(
            "4 0 obj\n<< /Type /ObjStm /N 3 /First {} /Filter /FlateDecode /Length {} >>\nstream\n",
            header.len(),
            objstm_data.len()
        )
        .as_bytes(),
    );
    out.extend_from_slice(&objstm_data);
    out.extend_from_slice(b"\nendstream\nendobj\n");
    let content_at = out.len();
    let stream = format!("BT 10 10 Td ({text}) Tj ET");
    out.extend_from_slice(
        format!(
            "5 0 obj\n<< /Length {} >>\nstream\n{stream}\nendstream\nendobj\n",
            stream.len()
        )
        .as_bytes(),
    );
    let xref_at = out.len();
    // Rows: type(1) field2(2) field3(1). Object 0 free, 1-3 in stream 4, 4 and 5 and 6 plain.
    let rows: [[u8; 4]; 7] = [
        [0, 0, 0, 255],
        [2, 0, 4, 0],
        [2, 0, 4, 1],
        [2, 0, 4, 2],
        [1, (objstm_at >> 8) as u8, objstm_at as u8, 0],
        [1, (content_at >> 8) as u8, content_at as u8, 0],
        [1, (xref_at >> 8) as u8, xref_at as u8, 0],
    ];
    let mut filtered = Vec::new();
    let mut previous = [0u8; 4];
    for row in rows {
        filtered.push(2);
        for i in 0..4 {
            filtered.push(row[i].wrapping_sub(previous[i]));
        }
        previous = row;
    }
    let xref_data = compress_to_vec_zlib(&filtered, 6);
    out.extend_from_slice(
        format!(
            "6 0 obj\n<< /Type /XRef /Size 7 /W [1 2 1] /Root 1 0 R /Filter /FlateDecode /DecodeParms << /Predictor 12 /Columns 4 >> /Length {} >>\nstream\n",
            xref_data.len()
        )
        .as_bytes(),
    );
    out.extend_from_slice(&xref_data);
    out.extend_from_slice(format!("\nendstream\nendobj\nstartxref\n{xref_at}\n%%EOF\n").as_bytes());
    out
}

fn contains(hay: &[u8], needle: &str) -> bool {
    hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

/// Byte-level replacement; keep `to` the same length as `from` unless it lies after the last object.
fn patch(pdf: &[u8], from: &str, to: &str) -> Vec<u8> {
    let at = pdf
        .windows(from.len())
        .position(|w| w == from.as_bytes())
        .expect("pattern present");
    [&pdf[..at], to.as_bytes(), &pdf[at + from.len()..]].concat()
}

/// Content-stream text of every page, following /Kids order.
fn page_texts(pdf: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(pdf).to_string();
    let kids = text
        .split("/Kids [")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap();
    kids.split(" R")
        .filter_map(|r| r.trim().split(' ').next().filter(|n| !n.is_empty()))
        .map(|page| {
            let body = text.split(&format!("\n{page} 0 obj\n")).nth(1).unwrap();
            let content = body
                .split("/Contents ")
                .nth(1)
                .unwrap()
                .split(' ')
                .next()
                .unwrap();
            let stream = text.split(&format!("\n{content} 0 obj\n")).nth(1).unwrap();
            let data = stream.split("stream\n").nth(1).unwrap();
            data.split("\nendstream").next().unwrap().to_string()
        })
        .collect()
}

#[test]
fn merges_two_classic_documents_in_order() {
    let a = classic(&["alpha one", "alpha two"]);
    let b = classic(&["beta one"]);
    let (merged, pages) = pdf::merge(&a, &b, 1 << 20).unwrap();
    assert_eq!(pages, 3);
    assert_eq!(pdf::page_count(&merged), Ok(3));
    let texts = page_texts(&merged);
    assert_eq!(texts.len(), 3);
    for (page, expected) in texts.iter().zip(["alpha one", "alpha two", "beta one"]) {
        assert!(page.contains(expected), "{page} should contain {expected}");
    }
    let text = String::from_utf8_lossy(&merged).to_string();
    // Inherited attributes are copied onto each page before the old tree is dropped.
    assert_eq!(text.matches("/MediaBox [0 0 200 100]").count(), 3);
    assert_eq!(text.matches("/Parent 2 0 R").count(), 3);
    assert!(text.starts_with("%PDF-1.7"));
}

#[test]
fn merging_is_stable_when_repeated() {
    let a = classic(&["a"]);
    let b = classic(&["b"]);
    let (once, _) = pdf::merge(&a, &b, 1 << 20).unwrap();
    let (again, pages) = pdf::merge(&once, &a, 1 << 20).unwrap();
    assert_eq!(pages, 3);
    assert_eq!(pdf::page_count(&again), Ok(3));
}

#[test]
fn reads_object_streams_and_xref_streams() {
    let a = modern("modern text");
    assert_eq!(pdf::page_count(&a), Ok(1));
    let (merged, pages) = pdf::merge(&a, &classic(&["classic text"]), 1 << 20).unwrap();
    assert_eq!(pages, 2);
    assert!(contains(&merged, "modern text"));
    assert!(contains(&merged, "/MediaBox [0 0 300 300]"));
    assert!(contains(&merged, "/MediaBox [0 0 200 100]"));
    // Objects that lived in the object stream are written as plain objects.
    assert!(!contains(&merged, "/ObjStm"));
    assert_eq!(pdf::page_count(&merged), Ok(2));
}

#[test]
fn rejects_unreadable_input() {
    let good = classic(&["x"]);
    for bad in [
        &b""[..],
        b"not a pdf",
        b"%PDF-1.4\nstartxref\n0\n%%EOF",
        &good[..good.len() / 2],
    ] {
        assert_eq!(pdf::merge(bad, &good, 1 << 20).unwrap_err(), "INVALID_PDF");
        assert_eq!(pdf::merge(&good, bad, 1 << 20).unwrap_err(), "INVALID_PDF");
    }
}

#[test]
fn rejects_encrypted_documents() {
    let plain = classic(&["x"]);
    let locked = patch(
        &plain,
        "/Root 1 0 R >>",
        "/Root 1 0 R /Encrypt << /Filter /Standard >> >>",
    );
    assert_eq!(
        pdf::merge(&locked, &plain, 1 << 20).unwrap_err(),
        "ENCRYPTED"
    );
    assert_eq!(
        pdf::merge(&plain, &locked, 1 << 20).unwrap_err(),
        "ENCRYPTED"
    );
}

#[test]
fn rejects_documents_without_pages() {
    let plain = classic(&["x"]);
    let empty = patch(&plain, "/Kids [4 0 R]", "/Kids []     ");
    assert_eq!(pdf::merge(&empty, &plain, 1 << 20).unwrap_err(), "NO_PAGES");
}

#[test]
fn refuses_output_beyond_the_limit() {
    let a = classic(&["a"]);
    assert_eq!(pdf::merge(&a, &a, 300).unwrap_err(), "TOO_LARGE");
}

#[test]
fn survives_hostile_page_trees() {
    let plain = classic(&["x"]);
    let looped = patch(&plain, "/Kids [4 0 R]", "/Kids [2 0 R]");
    assert!(pdf::merge(&looped, &plain, 1 << 20).is_err());
    let dangling = patch(&plain, "/Contents 5 0 R", "/Contents 9 0 R");
    let (merged, pages) = pdf::merge(&dangling, &plain, 1 << 20).unwrap();
    assert_eq!(pages, 2);
    assert!(contains(&merged, "/Contents null"));
}

#[test]
fn action_round_trips_through_base64() {
    let out = merge_pdfs(&json!({
        "first_base64":STANDARD.encode(classic(&["one", "two"])),
        "second_base64":STANDARD.encode(modern("three")),
    }))
    .unwrap();
    assert_eq!(out["pages"], 3);
    let merged = STANDARD
        .decode(out["pdf_base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(pdf::page_count(&merged), Ok(3));
}

#[test]
fn action_validates_input() {
    let ok = STANDARD.encode(classic(&["x"]));
    assert_eq!(
        merge_pdfs(&json!({"first_base64":ok})).unwrap_err(),
        "INVALID_INPUT"
    );
    assert_eq!(
        merge_pdfs(&json!({"first_base64":ok,"second_base64":"%%%"})).unwrap_err(),
        "INVALID_BASE64"
    );
    assert_eq!(
        merge_pdfs(&json!({"first_base64":ok,"second_base64":ok,"extra":1})).unwrap_err(),
        "INVALID_INPUT"
    );
    let huge = STANDARD.encode(vec![0u8; FILE_LIMIT as usize + 1]);
    assert_eq!(
        merge_pdfs(&json!({"first_base64":huge,"second_base64":ok})).unwrap_err(),
        "TOO_LARGE"
    );
}
