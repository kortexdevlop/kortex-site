use kortex_abi::{Envelope, HostError, MAX_MESSAGE_BYTES, Reply, RequestKind, pointer};
use serde_json::{Value, json};
#[link(wasm_import_module = "kortex_v1")]
unsafe extern "C" {
    fn host_enqueue(ptr: i32, len: i32) -> i32;
}
#[unsafe(no_mangle)]
pub extern "C" fn kortex_alloc(len: i32) -> i32 {
    if len <= 0 || len as usize > MAX_MESSAGE_BYTES {
        return 0;
    }
    let allocation = vec![0u8; len as usize].into_boxed_slice();
    Box::into_raw(allocation) as *mut u8 as i32
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kortex_dealloc(ptr: i32, len: i32) {
    // ABI caller must return each live allocation exactly once with its original length.
    if ptr == 0 || len <= 0 {
        return;
    }
    unsafe {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            ptr as u32 as *mut u8,
            len as usize,
        )));
    }
}
fn error(id: String, code: &str) -> Reply {
    Reply {
        v: 1,
        request_id: id,
        ok: false,
        data: None,
        error: Some(HostError {
            code: code.into(),
            message: code.into(),
            retryable: false,
        }),
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn kortex_dispatch(ptr: i32, len: i32) -> i64 {
    let fallback = "00000000-0000-4000-8000-000000000000".to_string();
    let memory_len = core::arch::wasm32::memory_size::<0>() * 65536;
    let result = match pointer::checked_range(ptr as u32, len as u32, memory_len) {
        Err(_) => error(fallback, "INVALID_BUFFER"),
        Ok(_) => {
            let bytes =
                unsafe { std::slice::from_raw_parts(ptr as u32 as *const u8, len as u32 as usize) };
            match Envelope::decode(bytes) {
                Err(_) => error(fallback, "INVALID_JSON"),
                Ok(request) => {
                    let id = request.request_id;
                    let data: Result<Value, &str> = match request.kind {
                        RequestKind::Init => {
                            // Deliberately unrecognized, permission-free probe: host must reject enqueue.
                            let probe = b"{\"kind\":\"probe\"}";
                            let status =
                                unsafe { host_enqueue(probe.as_ptr() as i32, probe.len() as i32) };
                            Ok(json!({"abi_version":"1.0","enqueue_status":status}))
                        }
                        RequestKind::Render => Ok(
                            json!({"type":"text","key":"result","props":{"text":"Ready"},"children":[]}),
                        ),
                        RequestKind::ActionInvoke => {
                            if request.payload.get("action_id").and_then(Value::as_str)
                                != Some("multiply")
                            {
                                Err("UNKNOWN_ACTION")
                            } else {
                                match (
                                    request
                                        .payload
                                        .get("input")
                                        .and_then(|v| v.get("a"))
                                        .and_then(Value::as_i64),
                                    request
                                        .payload
                                        .get("input")
                                        .and_then(|v| v.get("b"))
                                        .and_then(Value::as_i64),
                                ) {
                                    (Some(a), Some(b)) => a
                                        .checked_mul(b)
                                        .map(|result| json!({"result":result}))
                                        .ok_or("OVERFLOW"),
                                    _ => Err("INVALID_INPUT"),
                                }
                            }
                        }
                        _ => Err("UNSUPPORTED_KIND"),
                    };
                    match data {
                        Ok(Value::Object(data)) => Reply {
                            v: 1,
                            request_id: id,
                            ok: true,
                            data: Some(data),
                            error: None,
                        },
                        Err(code) => error(id, code),
                        _ => unreachable!(),
                    }
                }
            }
        }
    };
    let bytes = result.encode().expect("bounded response");
    let len = bytes.len();
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8 as u32;
    pointer::pack(ptr, len as u32)
}
