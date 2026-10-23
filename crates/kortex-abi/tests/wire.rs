use kortex_abi::{pointer::*, wire::CodecError, *};
const REQUEST: &str =
    r#"{"v":1,"request_id":"00000000-0000-4000-8000-000000000001","kind":"render","payload":{}}"#;
#[test]
fn request_roundtrip() {
    let request = Envelope::decode(REQUEST.as_bytes()).unwrap();
    assert_eq!(request.kind, RequestKind::Render);
    assert_eq!(
        Envelope::decode(&request.encode().unwrap()).unwrap(),
        request
    );
}
#[test]
fn reply_roundtrip() {
    for json in [
        r#"{"v":1,"request_id":"00000000-0000-4000-8000-000000000001","ok":true,"data":{}}"#,
        r#"{"v":1,"request_id":"00000000-0000-4000-8000-000000000001","ok":false,"error":{"code":"CUSTOM","message":"failed","retryable":false}}"#,
    ] {
        let r = Reply::decode(json.as_bytes()).unwrap();
        assert_eq!(Reply::decode(&r.encode().unwrap()).unwrap(), r);
    }
}
#[test]
fn invalid_envelopes() {
    for input in [
        REQUEST.replace("render", "unknown"),
        REQUEST.replace("\"v\":1", "\"v\":2"),
        REQUEST.replace("\"payload\":{}", "\"payload\":[]"),
        REQUEST.replace("00000000-0000-4000-8000-000000000001", "not-uuid"),
    ] {
        assert!(Envelope::decode(input.as_bytes()).is_err());
    }
    assert!(matches!(Envelope::decode(&[]), Err(CodecError::Empty)));
    assert!(matches!(
        Envelope::decode(&vec![b' '; MAX_MESSAGE_BYTES + 1]),
        Err(CodecError::TooLarge)
    ));
    assert!(Envelope::decode(&[255]).is_err());
}
#[test]
fn contradictory_reply() {
    assert!(
        Reply::decode(br#"{"v":1,"request_id":"00000000-0000-4000-8000-000000000001","ok":false}"#)
            .is_err()
    );
}
#[test]
fn pointer_roundtrip() {
    for (p, l) in [(0, 0), (0, 1), (u32::MAX, u32::MAX), (0x80000000, 42)] {
        assert_eq!(unpack(pack(p, l)), (p, l));
    }
}
#[test]
fn pointer_bounds() {
    assert_eq!(
        checked_range(0, 0, MAX_MEMORY_BYTES),
        Err(PointerError::Empty)
    );
    assert_eq!(
        checked_range(0, (MAX_MESSAGE_BYTES + 1) as u32, MAX_MEMORY_BYTES),
        Err(PointerError::MessageTooLarge)
    );
    assert_eq!(
        checked_range(u32::MAX, 1, MAX_MEMORY_BYTES),
        Err(PointerError::Overflow)
    );
    assert_eq!(checked_range(10, 2, 11), Err(PointerError::OutOfBounds));
    assert_eq!(
        checked_range(0, 1, MAX_MEMORY_BYTES + 1),
        Err(PointerError::MemoryTooLarge)
    );
    assert_eq!(
        checked_range(0, MAX_MESSAGE_BYTES as u32, MAX_MEMORY_BYTES),
        Ok(0..MAX_MESSAGE_BYTES)
    );
}
#[test]
fn enqueue_codes() {
    assert_eq!(EnqueueStatus::try_from(0), Ok(EnqueueStatus::Accepted));
    assert_eq!(
        EnqueueStatus::try_from(-42),
        Ok(EnqueueStatus::Rejected(-42))
    );
    assert!(EnqueueStatus::try_from(1).is_err());
}
