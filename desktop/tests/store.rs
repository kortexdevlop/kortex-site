use kortex_desktop::{
    api::{Api, Error, Query},
    models::{ToolList, ToolSummary, decode_detail},
    store::Store,
};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};
fn item(i: usize) -> ToolSummary {
    serde_json::from_value(json!({"id":format!("item-{i:05}"),"version":"1.0.0","name":format!("Outil {i}"),"summary":"Un outil synthétique pour tester la pagination.","category":"developer","tags":[],"icon_path":"assets/icon.svg","abi_version":"1.0","sha256":"a".repeat(64),"published_at":1})).unwrap()
}
fn serve(responses: Vec<(u16, Value)>) -> (String, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let output = count.clone();
    let task = thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 16384];
            let n = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..n]);
            assert!(!request.contains("Authorization:"));
            output.fetch_add(1, Ordering::SeqCst);
            let text = body.to_string();
            write!(stream,"HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",text.len(),text).unwrap();
        }
    });
    (address, count, task)
}
#[test]
fn ten_thousand_entries_paginate_without_duplicates_and_scope_resets() {
    let mut store = Store::default();
    for start in (0..10000).step_by(100) {
        let end = (start + 100).min(10000);
        store
            .accept(ToolList {
                items: (start..end).map(item).collect(),
                next_cursor: if end < 10000 {
                    Some(format!("page-{end}"))
                } else {
                    None
                },
            })
            .unwrap();
    }
    assert_eq!(store.loaded, 10000);
    assert!(
        store
            .accept(ToolList {
                items: vec![item(9999)],
                next_cursor: None
            })
            .is_err()
    );
    store.reset(Query {
        q: "nouveau".into(),
        category: "image".into(),
        sort: "name".into(),
    });
    assert!(store.page.is_none());
    assert_eq!(store.loaded, 0);
    store
        .accept(ToolList {
            items: vec![item(0)],
            next_cursor: None,
        })
        .unwrap();
    assert!(
        store
            .accept(ToolList {
                items: vec![item(1), item(1)],
                next_cursor: None
            })
            .is_err()
    );
    let mut cyclic = Store::default();
    cyclic
        .accept(ToolList {
            items: vec![item(0)],
            next_cursor: Some("cycle".into()),
        })
        .unwrap();
    assert!(
        cyclic
            .accept(ToolList {
                items: vec![item(1)],
                next_cursor: Some("cycle".into())
            })
            .is_err()
    );
}
#[test]
fn requests_use_real_http_schema_validation_ttl_and_explicit_failures() {
    let page = json!({"items":[item(0)],"next_cursor":null});
    let (origin, count, task) = serve(vec![
        (200, page.clone()),
        (200, page.clone()),
        (404, json!({})),
        (503, json!({})),
        (200, json!({"items":[],"next_cursor":null,"injected":true})),
    ]);
    let api = Api::loopback_test(&origin);
    let q = Query::default();
    assert_eq!(api.list(&q, None, false).unwrap().items.len(), 1);
    assert_eq!(api.list(&q, None, false).unwrap().items.len(), 1);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    api.expire_test_cache();
    api.list(&q, None, false).unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 2);
    assert!(matches!(api.detail("missing"), Err(Error::NotFound)));
    assert!(matches!(api.list(&q, None, true), Err(Error::Unavailable)));
    assert!(matches!(api.list(&q, None, true), Err(Error::Invalid(_))));
    task.join().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let offline = Api::loopback_test(&format!("http://{}", listener.local_addr().unwrap()));
    drop(listener);
    assert!(matches!(offline.list(&q, None, true), Err(Error::Offline)));
}
#[test]
fn filters_are_encoded_and_hostile_ids_or_inconsistent_details_are_refused() {
    let q = Query {
        q: "été & image".into(),
        category: "image".into(),
        sort: "name".into(),
    };
    let pairs = q.params(Some("a+b/=")).unwrap();
    let encoded = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&pairs)
        .finish();
    assert!(encoded.contains("cursor=a%2Bb%2F%3D"));
    assert!(encoded.contains("q=%C3%A9t%C3%A9+%26+image"));
    assert!(
        Query {
            q: "a".repeat(201),
            ..Default::default()
        }
        .params(None)
        .is_err()
    );
    assert!(
        Query {
            category: "../x".into(),
            ..Default::default()
        }
        .params(None)
        .is_err()
    );
    assert!(q.params(Some("")).is_err());
    assert!(decode_detail(json!({}), "any").is_err());
}
