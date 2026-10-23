# Shared contract generation

The JSON Schemas in shared/schemas are the source of truth. TypeScript compiles AJV validators at import time; Rust embeds the identical schema documents with include_str and compiles validators lazily. No opaque validator source is checked in. Public model declarations are maintained explicitly and checked with the same positive/negative fixtures; types alone are not runtime validation.

Regenerate and verify with pnpm --filter @kortex/contracts test, then cargo test -p kortex-shared --locked. The test builds a deterministic JSON snapshot of all five current schemas in target/contracts/schemas.generated.json twice, compares bytes and hashes of original schemas, then compares the TypeScript/Rust acceptance matrix. Repeated runs must leave git diff unchanged. The snapshot is a derived diagnostic, not a second schema authority.

Rust exposes validate_manifest/validate_contract -> Result<model, Vec<Violation>>. TypeScript exposes validateManifest/validateContract/validateUi/validateWorkflow -> discriminated Result. Violation codes SCHEMA/INVARIANT/LIMIT are stable categories, never include input values. TypeScript includes AJV JSON Pointer paths; Rust currently uses the document root for schema failures. Message wording is deliberately canonical; equal diagnostics at every nested path are not claimed.

Additional tree/DAG invariants are checked outside JSON Schema. Rust UI/workflow validators are not promised by this step; their fixtures are tested by TypeScript while the cross-language matrix covers manifest and contract. Rust schema resolver HTTP and file features are disabled.
