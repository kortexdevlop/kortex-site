# Conformance corpus 1.0

Run from any directory: node tests/conformance/check.mjs (path relative to repository root in this example). The runner resolves fixtures relative to its own file. It consumes serialized message traces and expected results, validates JSON with the official schema and validates action data with the shared contracts.

| Surface | Local proof | Real-host proof |
|---|---|---|
| ABI exports/imports, pointers, memory | kortex-abi Rust tests | Phase 3 guest and host prototypes |
| JSON init/render/actions/errors | v1.cases.json, protocol schema | replay these cases against captured runtime traces |
| Lifecycle and request correlation | ordered trace oracle | Phase 3 and runtime implementation |
| Declared and granted permissions | admission cases, unknown capability rejected | mediated host capability tests |
| ABI/format/UI/contract and min runtime | negotiation oracle | unsupported ABI rejected before actual instantiation |

The runner is a specification oracle, not an executing tool or a mock presented as a runtime. ACCEPT means the captured exchange conforms; it does not prove that a host emitted it. Positive action data is schema-checked, not computed. Concurrent dispatch/reentrancy, real allocation ownership, asynchronous host queues and signature cryptography remain prototype/runtime obligations.

Changes to corpus cases must cite a contract rule and an approved step. A frozen case must never be silently rewritten to make an incompatible runtime pass. Human approval is recorded in PROGRESS; signed release/key changes follow the trust contract. This repository currently has no configured signing identity: no cryptographic signature of this initial corpus is claimed.
