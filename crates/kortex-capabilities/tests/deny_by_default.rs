use kortex_capabilities::{
    handles::Access,
    policy::{Context, Decision, Duration},
    *,
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    net::IpAddr,
    rc::Rc,
};
fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(future);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    for _ in 0..1000 {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
    }
    panic!("test future pending")
}
fn owner(id: &str, instance: u64) -> Context {
    Context {
        tool_id: id.into(),
        version: "1.0.0".into(),
        instance_id: instance,
    }
}
fn request(capability: Capability, resource: Resource) -> CapabilityCall {
    CapabilityCall {
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        capability,
        resource,
        bytes: Vec::new(),
        max_output_bytes: 1024,
    }
}
struct Host {
    calls: Rc<Cell<usize>>,
    resources: Rc<RefCell<Vec<Resource>>>,
    responses: VecDeque<CapabilityResponse>,
    ips: Vec<IpAddr>,
    clock: Option<Rc<Cell<u64>>>,
}
impl CapabilityHost for Host {
    async fn resolve(&mut self, _: &str) -> Result<Vec<IpAddr>, CapabilityError> {
        if let Some(clock) = &self.clock {
            clock.set(1001)
        }
        Ok(self.ips.clone())
    }
    async fn execute(
        &mut self,
        permit: &AuthorizedCall<'_>,
    ) -> Result<CapabilityResponse, CapabilityError> {
        self.calls.set(self.calls.get() + 1);
        self.resources
            .borrow_mut()
            .push(permit.request().resource.clone());
        Ok(self
            .responses
            .pop_front()
            .unwrap_or(CapabilityResponse::Bytes(b"ok".to_vec())))
    }
}
fn make_gateway(responses: Vec<CapabilityResponse>) -> (Gateway<Host>, Rc<Cell<usize>>) {
    let count = Rc::new(Cell::new(0));
    (
        Gateway::new(Host {
            calls: count.clone(),
            resources: Default::default(),
            responses: responses.into(),
            ips: vec!["93.184.216.34".parse().unwrap()],
            clock: None,
        }),
        count,
    )
}
fn storage() -> Resource {
    Resource::Storage {
        key: "preferences".into(),
    }
}
fn network(value: &str) -> Resource {
    Resource::Network { url: value.into() }
}
#[test]
fn dns_rebinding_after_same_origin_redirect_is_denied_before_second_effect() {
    struct Rebinding {
        queries: usize,
        calls: Rc<Cell<usize>>,
    }
    impl CapabilityHost for Rebinding {
        async fn resolve(&mut self, _: &str) -> Result<Vec<IpAddr>, CapabilityError> {
            self.queries += 1;
            Ok(vec![
                if self.queries == 1 {
                    "93.184.216.34"
                } else {
                    "127.0.0.1"
                }
                .parse()
                .unwrap(),
            ])
        }
        async fn execute(
            &mut self,
            call: &AuthorizedCall<'_>,
        ) -> Result<CapabilityResponse, CapabilityError> {
            assert_eq!(
                call.addresses(),
                &["93.184.216.34".parse::<IpAddr>().unwrap()]
            );
            self.calls.set(self.calls.get() + 1);
            Ok(CapabilityResponse::Redirect("/next".into()))
        }
    }
    let calls = Rc::new(Cell::new(0));
    let mut g = Gateway::new(Rebinding {
        queries: 0,
        calls: calls.clone(),
    });
    let a = owner("tool-a", 1);
    let resource = network("https://allowed.example/start");
    g.policy.declare(a.clone(), [Capability::NetworkFetch]);
    g.policy
        .grant(
            &a,
            Capability::NetworkFetch,
            &resource,
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(matches!(
        block_on(g.execute(&a, request(Capability::NetworkFetch, resource), None, &|| 0)),
        Err(CapabilityError::NetworkDenied)
    ));
    assert_eq!(calls.get(), 1);
}
#[test]
fn deny_is_default_and_undeclared_cannot_be_granted() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    assert_eq!(
        gateway
            .policy
            .authorize(&a, Capability::StorageRead, &storage(), None, 0),
        Decision::Deny
    );
    assert_eq!(
        gateway.policy.grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Session,
            1000
        ),
        Err(CapabilityError::Denied)
    );
    assert!(matches!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0)),
        Err(CapabilityError::Denied)
    ));
    assert_eq!(calls.get(), 0);
    assert!(Capability::try_from("gpu.direct").is_err());
}
#[test]
fn ask_requires_explicit_prompting_and_never_executes() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    gateway.policy.declare(a.clone(), [Capability::StorageRead]);
    assert_eq!(
        gateway
            .policy
            .authorize(&a, Capability::StorageRead, &storage(), None, 0),
        Decision::Deny
    );
    gateway.policy.enable_prompting(&a);
    assert_eq!(
        gateway
            .policy
            .authorize(&a, Capability::StorageRead, &storage(), None, 0),
        Decision::Ask
    );
    assert!(matches!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0)),
        Err(CapabilityError::NeedsConsent)
    ));
    let prompt = gateway
        .consent_prompt(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Once,
            1000,
        )
        .unwrap();
    assert_eq!(prompt.scope, "storage:preferences");
    assert_eq!(calls.get(), 0);
}
#[test]
fn once_consumed_session_expiry_and_revocation() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    gateway.policy.declare(a.clone(), [Capability::StorageRead]);
    gateway
        .policy
        .grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Once,
            1000,
        )
        .unwrap();
    assert!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0))
            .is_ok()
    );
    assert!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0))
            .is_err()
    );
    gateway
        .policy
        .grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::StorageRead, storage()),
            None,
            &|| 999
        ))
        .is_ok()
    );
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::StorageRead, storage()),
            None,
            &|| 1000
        ))
        .is_err()
    );
    gateway.revoke_instance(&a);
    assert!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0))
            .is_err()
    );
    assert_eq!(calls.get(), 2);
}
#[test]
fn grant_cannot_cross_tool_version_instance_or_resource() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    gateway.policy.declare(a.clone(), [Capability::StorageRead]);
    gateway
        .policy
        .grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Session,
            1000,
        )
        .unwrap();
    for b in [
        owner("tool-b", 1),
        Context {
            version: "1.0.1".into(),
            ..a.clone()
        },
        owner("tool-a", 2),
    ] {
        gateway.policy.declare(b.clone(), [Capability::StorageRead]);
        assert!(
            block_on(gateway.execute(&b, request(Capability::StorageRead, storage()), None, &|| 0))
                .is_err()
        );
    }
    assert!(
        block_on(gateway.execute(
            &a,
            request(
                Capability::StorageRead,
                Resource::Storage {
                    key: "other".into()
                }
            ),
            None,
            &|| 0
        ))
        .is_err()
    );
    assert_eq!(calls.get(), 0);
}
#[test]
fn clipboard_requires_fresh_single_use_host_gesture() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    gateway
        .policy
        .declare(a.clone(), [Capability::ClipboardWrite]);
    gateway
        .policy
        .grant(
            &a,
            Capability::ClipboardWrite,
            &Resource::Clipboard,
            Duration::Session,
            5000,
        )
        .unwrap();
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::ClipboardWrite, Resource::Clipboard),
            None,
            &|| 0
        ))
        .is_err()
    );
    let gesture = gateway.policy.record_gesture(&a, 0).unwrap();
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::ClipboardWrite, Resource::Clipboard),
            Some(&gesture),
            &|| 0
        ))
        .is_ok()
    );
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::ClipboardWrite, Resource::Clipboard),
            Some(&gesture),
            &|| 0
        ))
        .is_err()
    );
    let expired = gateway.policy.record_gesture(&a, 0).unwrap();
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::ClipboardWrite, Resource::Clipboard),
            Some(&expired),
            &|| 1000
        ))
        .is_err()
    );
    assert_eq!(calls.get(), 1);
}
#[test]
fn opaque_handles_are_short_lived_and_owner_bound() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    let handle = gateway
        .handles
        .issue(
            a.clone(),
            Access::Read,
            "private-backend-path".into(),
            0,
            100,
        )
        .unwrap();
    assert_eq!(handle.len(), 64);
    assert!(!handle.contains("private"));
    let resource = Resource::File {
        handle: handle.clone(),
    };
    gateway.policy.declare(a.clone(), [Capability::FileRead]);
    gateway
        .policy
        .grant(&a, Capability::FileRead, &resource, Duration::Session, 1000)
        .unwrap();
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::FileRead, resource.clone()),
            None,
            &|| 0
        ))
        .is_ok()
    );
    assert!(matches!(
        block_on(gateway.execute(
            &a,
            request(Capability::FileRead, resource.clone()),
            None,
            &|| 100
        )),
        Err(CapabilityError::InvalidHandle)
    ));
    let b = owner("tool-b", 2);
    gateway.policy.declare(b.clone(), [Capability::FileRead]);
    gateway
        .policy
        .grant(&b, Capability::FileRead, &resource, Duration::Session, 1000)
        .unwrap();
    assert!(matches!(
        block_on(gateway.execute(&b, request(Capability::FileRead, resource), None, &|| 0)),
        Err(CapabilityError::InvalidHandle)
    ));
    assert_eq!(calls.get(), 1);
}
#[test]
fn file_and_response_budgets_are_enforced() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    let mut oversized = request(Capability::FileSave, Resource::FileDialog);
    oversized.bytes = vec![0; 10 * 1024 * 1024 + 1];
    assert!(matches!(
        block_on(gateway.execute(&a, oversized, None, &|| 0)),
        Err(CapabilityError::SizeLimit)
    ));
    assert_eq!(calls.get(), 0);
    gateway.policy.declare(a.clone(), [Capability::StorageRead]);
    gateway
        .policy
        .grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Session,
            1000,
        )
        .unwrap();
    let mut small = request(Capability::StorageRead, storage());
    small.max_output_bytes = 1;
    assert!(matches!(
        block_on(gateway.execute(&a, small, None, &|| 0)),
        Err(CapabilityError::SizeLimit)
    ));
    assert_eq!(calls.get(), 1);
}
#[test]
fn redirect_to_unapproved_origin_is_rejected_before_effect() {
    let (mut gateway, calls) = make_gateway(vec![CapabilityResponse::Redirect(
        "https://evil.example/target".into(),
    )]);
    let a = owner("tool-a", 1);
    let resource = network("https://allowed.example/start");
    gateway
        .policy
        .declare(a.clone(), [Capability::NetworkFetch]);
    gateway
        .policy
        .grant(
            &a,
            Capability::NetworkFetch,
            &resource,
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(matches!(
        block_on(gateway.execute(&a, request(Capability::NetworkFetch, resource), None, &|| 0)),
        Err(CapabilityError::NetworkDenied)
    ));
    assert_eq!(calls.get(), 1);
}
#[test]
fn same_origin_redirect_is_bounded_and_reuses_once_permission() {
    let (mut gateway, calls) = make_gateway(vec![
        CapabilityResponse::Redirect("/next".into()),
        CapabilityResponse::Bytes(b"ok".to_vec()),
    ]);
    let a = owner("tool-a", 1);
    let resource = network("https://allowed.example/start");
    gateway
        .policy
        .declare(a.clone(), [Capability::NetworkFetch]);
    gateway
        .policy
        .grant(
            &a,
            Capability::NetworkFetch,
            &resource,
            Duration::Once,
            1000,
        )
        .unwrap();
    assert!(
        block_on(gateway.execute(
            &a,
            request(Capability::NetworkFetch, resource.clone()),
            None,
            &|| 0
        ))
        .is_ok()
    );
    assert!(
        block_on(gateway.execute(&a, request(Capability::NetworkFetch, resource), None, &|| 0))
            .is_err()
    );
    assert_eq!(calls.get(), 2);
}
#[test]
fn private_dns_and_ip_forms_are_denied() {
    for target in [
        "http://127.0.0.1/",
        "http://2130706433/",
        "http://[::1]/",
        "http://[::ffff:127.0.0.1]/",
        "http://192.168.1.1/",
    ] {
        let (mut gateway, calls) = make_gateway(vec![]);
        let a = owner("tool-a", 1);
        let resource = network(target);
        gateway
            .policy
            .declare(a.clone(), [Capability::NetworkFetch]);
        gateway
            .policy
            .grant(
                &a,
                Capability::NetworkFetch,
                &resource,
                Duration::Session,
                1000,
            )
            .unwrap();
        assert!(matches!(
            block_on(gateway.execute(&a, request(Capability::NetworkFetch, resource), None, &|| 0)),
            Err(CapabilityError::NetworkDenied)
        ));
        assert_eq!(calls.get(), 0);
    }
}
#[test]
fn expiry_during_dns_is_rechecked_before_host_effect() {
    let count = Rc::new(Cell::new(0));
    let time = Rc::new(Cell::new(0));
    let host = Host {
        calls: count.clone(),
        resources: Default::default(),
        responses: VecDeque::new(),
        ips: vec!["93.184.216.34".parse().unwrap()],
        clock: Some(time.clone()),
    };
    let mut gateway = Gateway::new(host);
    let a = owner("tool-a", 1);
    let resource = network("https://allowed.example/");
    gateway
        .policy
        .declare(a.clone(), [Capability::NetworkFetch]);
    gateway
        .policy
        .grant(
            &a,
            Capability::NetworkFetch,
            &resource,
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(matches!(
        block_on(gateway.execute(
            &a,
            request(Capability::NetworkFetch, resource),
            None,
            &|| time.get()
        )),
        Err(CapabilityError::Denied)
    ));
    assert_eq!(count.get(), 0);
}
#[test]
fn audit_is_local_bounded_and_contains_no_sensitive_data() {
    let (mut gateway, _) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    let resource = Resource::Storage {
        key: "secret-file-name".into(),
    };
    for _ in 0..1030 {
        let mut call = request(Capability::StorageRead, resource.clone());
        call.bytes = b"private-payload https://private.example".to_vec();
        let _ = block_on(gateway.execute(&a, call, None, &|| 0));
    }
    let events = gateway.audit.events().collect::<Vec<_>>();
    assert_eq!(events.len(), 1024);
    let text = format!("{events:?}");
    for forbidden in ["secret-file-name", "private-payload", "private.example"] {
        assert!(!text.contains(forbidden));
    }
}

#[test]
fn all_seven_capabilities_require_policy_before_host_call() {
    for cap in [
        Capability::FilePick,
        Capability::FileRead,
        Capability::FileSave,
        Capability::StorageRead,
        Capability::StorageWrite,
        Capability::NetworkFetch,
        Capability::ClipboardWrite,
    ] {
        let (mut gateway, calls) = make_gateway(vec![]);
        let a = owner("tool-a", 1);
        let resource = match cap {
            Capability::FilePick | Capability::FileSave => Resource::FileDialog,
            Capability::FileRead => Resource::File {
                handle: gateway
                    .handles
                    .issue(a.clone(), Access::Read, "native-ref".into(), 0, 1000)
                    .unwrap(),
            },
            Capability::StorageRead | Capability::StorageWrite => storage(),
            Capability::NetworkFetch => network("https://allowed.example/"),
            Capability::ClipboardWrite => Resource::Clipboard,
        };
        gateway.policy.declare(a.clone(), [cap]);
        gateway
            .policy
            .grant(&a, cap, &resource, Duration::Session, 1000)
            .unwrap();
        let gesture = if cap.needs_gesture() {
            Some(gateway.policy.record_gesture(&a, 0).unwrap())
        } else {
            None
        };
        assert!(
            block_on(gateway.execute(&a, request(cap, resource), gesture.as_ref(), &|| 0)).is_ok()
        );
        assert_eq!(calls.get(), 1);
    }
}
#[test]
fn explicitly_authorized_redirect_succeeds_and_loops_stop() {
    let (mut gateway, calls) = make_gateway(vec![
        CapabilityResponse::Redirect("https://other.example/path".into()),
        CapabilityResponse::Bytes(b"ok".to_vec()),
    ]);
    let a = owner("tool-a", 1);
    gateway
        .policy
        .declare(a.clone(), [Capability::NetworkFetch]);
    for resource in [
        network("https://allowed.example/"),
        network("https://other.example/"),
    ] {
        gateway
            .policy
            .grant(
                &a,
                Capability::NetworkFetch,
                &resource,
                Duration::Session,
                1000,
            )
            .unwrap();
    }
    assert!(
        block_on(gateway.execute(
            &a,
            request(
                Capability::NetworkFetch,
                network("https://allowed.example/")
            ),
            None,
            &|| 0
        ))
        .is_ok()
    );
    assert_eq!(calls.get(), 2);
    let (mut gateway, calls) = make_gateway(
        (0..6)
            .map(|_| CapabilityResponse::Redirect("/loop".into()))
            .collect(),
    );
    gateway
        .policy
        .declare(a.clone(), [Capability::NetworkFetch]);
    gateway
        .policy
        .grant(
            &a,
            Capability::NetworkFetch,
            &network("https://allowed.example/"),
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(matches!(
        block_on(gateway.execute(
            &a,
            request(
                Capability::NetworkFetch,
                network("https://allowed.example/")
            ),
            None,
            &|| 0
        )),
        Err(CapabilityError::RedirectLimit)
    ));
    assert_eq!(calls.get(), 6);
}
#[test]
fn private_dns_answer_is_denied_before_connection() {
    let calls = Rc::new(Cell::new(0));
    let host = Host {
        calls: calls.clone(),
        resources: Default::default(),
        responses: VecDeque::new(),
        ips: vec!["127.0.0.1".parse().unwrap()],
        clock: None,
    };
    let mut gateway = Gateway::new(host);
    let a = owner("tool-a", 1);
    let resource = network("https://allowed.example/");
    gateway
        .policy
        .declare(a.clone(), [Capability::NetworkFetch]);
    gateway
        .policy
        .grant(
            &a,
            Capability::NetworkFetch,
            &resource,
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(matches!(
        block_on(gateway.execute(&a, request(Capability::NetworkFetch, resource), None, &|| 0)),
        Err(CapabilityError::NetworkDenied)
    ));
    assert_eq!(calls.get(), 0);
}

#[test]
fn explicit_deny_overrides_session_and_stops_reprompting() {
    let (mut gateway, calls) = make_gateway(vec![]);
    let a = owner("tool-a", 1);
    gateway.policy.declare(a.clone(), [Capability::StorageRead]);
    gateway.policy.enable_prompting(&a);
    gateway
        .policy
        .grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Session,
            1000,
        )
        .unwrap();
    gateway
        .policy
        .deny(&a, Capability::StorageRead, &storage())
        .unwrap();
    assert_eq!(
        gateway
            .policy
            .authorize(&a, Capability::StorageRead, &storage(), None, 0),
        Decision::Deny
    );
    assert!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0))
            .is_err()
    );
    assert_eq!(calls.get(), 0);
    gateway
        .policy
        .grant(
            &a,
            Capability::StorageRead,
            &storage(),
            Duration::Once,
            1000,
        )
        .unwrap();
    assert!(
        block_on(gateway.execute(&a, request(Capability::StorageRead, storage()), None, &|| 0))
            .is_ok()
    );
}
