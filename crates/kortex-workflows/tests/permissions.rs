mod common;
use common::*;
use kortex_workflows::{
    permissions::preview,
    runner::{Cancellation, Options, Run},
    *,
};
use serde_json::json;
#[test]
fn no_step_launches_without_workflow_consent_or_actual_policy_grant() {
    let reg = registry(&["storage.write"], true, "local-write");
    let compiled = Compiled::new(&workflow(), &reg).unwrap();
    assert_eq!(
        kortex_workflows::permissions::aggregated_permissions(&compiled),
        vec!["storage.write"]
    );
    assert!(preview(&compiled).iter().all(|p| p.confirmation));
    let mut run = Run::new(compiled, Options::default()).unwrap();
    let mut consent = run.consent(0, true);
    let mut host = TestHost::new(reg);
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut consent,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::AwaitingConsent
    );
    consent
        .approve_batch(run.plan(), &["a".into(), "b".into()])
        .unwrap();
    assert_eq!(run.pending_node().unwrap().id, "a");
    assert_eq!(run.journal()[0].status, Error::AwaitingConsent);
    assert!(!run.journal()[0].may_have_effects);
    host.policy.declare(
        host.registry
            .get("calculatrice.add")
            .unwrap()
            .context
            .clone(),
        [kortex_capabilities::Capability::StorageWrite],
    );
    host.policy
        .enable_prompting(&host.registry.get("calculatrice.add").unwrap().context);
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut consent,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::AwaitingConsent
    );
    assert!(host.calls.is_empty());
    host.grant();
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut consent,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Completed
    );
    assert_eq!(host.calls, vec!["a", "b"]);
}
#[test]
fn revocation_during_effect_stops_next_node_without_claiming_rollback() {
    let reg = registry(&["storage.write"], true, "local-write");
    let mut run = Run::new(
        Compiled::new(&workflow(), &reg).unwrap(),
        Options::default(),
    )
    .unwrap();
    let mut c = run.consent(0, false);
    c.approve_batch(run.plan(), &["a".into(), "b".into()])
        .unwrap();
    let mut host = TestHost::new(reg);
    host.grant();
    host.revoke = true;
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Cancelled
    );
    assert_eq!(host.calls, vec!["a"]);
    assert!(run.journal()[0].may_have_effects);
    assert!(run.outputs().contains_key("a"));
}
#[test]
fn node_approval_resumes_without_replaying_completed_nodes_and_batch_cannot_be_reused() {
    let reg = registry(&[], true, "none");
    let mut w = workflow();
    for n in w["nodes"].as_array_mut().unwrap() {
        n["requires_confirmation"] = json!(true)
    }
    let mut run = Run::new(Compiled::new(&w, &reg).unwrap(), Options::default()).unwrap();
    let mut c = run.consent(0, true);
    c.approve_node(run.plan(), "a").unwrap();
    let mut host = TestHost::new(reg.clone());
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::AwaitingConsent
    );
    assert_eq!(host.calls, vec!["a"]);
    c.approve_node(run.plan(), "b").unwrap();
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Completed
    );
    assert!(c.approve_node(run.plan(), "a").is_err());
    let mut other = Run::new(Compiled::new(&w, &reg).unwrap(), Options::default()).unwrap();
    assert_eq!(
        block(other.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::PermissionDenied
    );
    assert_eq!(host.calls, vec!["a", "b"]);
}
#[test]
fn changed_plan_or_registry_never_inherits_consent() {
    let reg = registry(&[], true, "none");
    let mut run = Run::new(
        Compiled::new(&workflow(), &reg).unwrap(),
        Options::default(),
    )
    .unwrap();
    let mut c = run.consent(0, false);
    let mut different = workflow();
    different["nodes"][0]["input_bindings"]["/a"] = json!({"value":99});
    assert!(
        c.approve_node(&Compiled::new(&different, &reg).unwrap(), "a")
            .is_err()
    );
    let mut host = TestHost::new(registry(&["storage.read"], true, "none"));
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Changed
    );
    assert!(host.calls.is_empty());
}
#[test]
fn revoke_while_awaiting_and_expired_consent_stop_before_invocation() {
    let reg = registry(&[], true, "none");
    let mut run = Run::new(
        Compiled::new(&workflow(), &reg).unwrap(),
        Options::default(),
    )
    .unwrap();
    let mut c = run.consent(0, false);
    let mut host = TestHost::new(reg);
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::AwaitingConsent
    );
    host.policy
        .revoke_instance(&host.registry.get("calculatrice.add").unwrap().context);
    c.approve_node(run.plan(), "a").unwrap();
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Cancelled
    );
    assert!(host.calls.is_empty());
    let reg = registry(&[], true, "none");
    let mut run = Run::new(
        Compiled::new(&workflow(), &reg).unwrap(),
        Options::default(),
    )
    .unwrap();
    let mut c = run.consent(0, true);
    let mut host = TestHost::new(reg);
    let clock = TestClock::default();
    clock.now.set(300000);
    assert_eq!(
        block(run.advance(&mut host, &mut c, &clock, &Cancellation::default())),
        Error::Expired
    );
    assert!(host.calls.is_empty());
}
#[test]
fn cancellation_and_timeout_drop_pending_work_before_any_next_node() {
    for cancel_mode in [true, false] {
        let reg = registry(&[], true, "none");
        let mut run = Run::new(
            Compiled::new(&workflow(), &reg).unwrap(),
            Options::default(),
        )
        .unwrap();
        let mut c = run.consent(0, true);
        let mut host = TestHost::new(reg);
        host.hang = true;
        let token = Cancellation::default();
        if cancel_mode {
            host.cancel = Some(token.clone())
        }
        let clock = TestClock {
            expire: !cancel_mode,
            ..Default::default()
        };
        assert_eq!(
            block(run.advance(&mut host, &mut c, &clock, &token)),
            if cancel_mode {
                Error::Cancelled
            } else {
                Error::Timeout
            }
        );
        assert!(host.dropped.get());
        assert_eq!(host.calls, vec!["a"]);
    }
}
