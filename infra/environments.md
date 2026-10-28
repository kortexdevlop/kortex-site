# Environments and ownership

## Identity checkpoint — 2026-10-28, step 14.0

Owner selected Google; OAuth clients do not yet exist. Public staging intent: infra/identity/google-staging.json, disabled with null client IDs. Issuer: https://accounts.google.com. Explicit endpoint origins: accounts.google.com, oauth2.googleapis.com, www.googleapis.com. Actual discovery/JWKS/S256 checked; no OAuth client or login claimed.

Configuration consumes OIDC_ENABLED, OIDC_ISSUER, OIDC_WEB_CLIENT_ID, OIDC_DESKTOP_CLIENT_ID, OIDC_WEB_REDIRECT_URI, OIDC_DESKTOP_REDIRECT_URI and comma-separated OIDC_ENDPOINT_ORIGINS. These are server/host settings, never arbitrary request input. Web callback: https://kortex-web-staging.userpgl.workers.dev/auth/callback ; Desktop: http://127.0.0.1:49152/callback. OIDC_WEB_CLIENT_SECRET and SESSION_HMAC_SECRET (at least 32 random bytes) will be server secrets for the later login implementation; never PUBLIC_* or tracked configuration. Separate production clients unprovisioned. See docs/AUTH.md for setup and resume. Migration 0005_auth.sql tested locally ; no production identity deployment.

| Environment | Web / API / runtime | Cloudflare resources | Identity | Ownership |
|---|---|---|---|---|
| local | localhost:4321 / :8787 / :4322 | local emulation only | disabled until configured | developer |
| staging | separate HTTPS origins, values pending | dedicated D1, R2, Worker, account IDs pending | dedicated OIDC client pending | project owner provisions; CI deploys with scoped permission |
| production | separate HTTPS origins, values pending | separate D1, R2, Worker; IDs pending | production OIDC client pending | project owner approves; protected release CI deploys |

Actual owner names, resource IDs and domains must be supplied by the project owner. No cloud resource is provisioned by this step. Never reuse production storage or credentials in staging.

PUBLIC_* permits only the three origins in the configuration checker. OIDC_CLIENT_ID and KORTEX_SIGNING_PUBLIC_KEY are non-secret but are not automatically browser-exposed. OIDC_CLIENT_SECRET remains server-side when required by the selected provider. Public-client PKCE does not assume a secret.

KORTEX_SIGNING_PRIVATE_KEY is exclusively a protected CI secret for artifact signing. It must not appear in application environment files, browser bundles, logs or fixtures. CI uses narrowly scoped identities/OIDC where available. Cloudflare credentials belong in protected GitHub/Cloudflare settings, never PUBLIC_*. Signing public keys and key IDs are distributed separately from private keys.

Copy .env.example to .env for local configuration. For staging/production inject values into the process or platform secrets. Run node scripts/check-env.mjs before startup/deployment. Required non-local metadata is checked without displaying values. This structural check does not verify remote resource existence or cryptographic key validity; those checks belong to provisioning and signing steps.

## Catalog binding planned at step 9.0

The API will expose D1 through binding `KORTEX_DB`, separately provisioned for staging and production. Real database names/IDs and the API Wrangler configuration remain pending at step 9.2. Do not substitute the all-zero local-test UUID for a remote resource. Step 9.0 runs migration 0001 and 10,000 synthetic tools through SQLite and isolated Wrangler D1 **local** state under target/ only; no account credentials or remote write are needed. Migrations live in database/migrations; Wrangler tracks applied scripts in d1_migrations. See docs/DATABASE.md. R2 integrity/publication state follows at 9.1; artifacts are never stored in D1.

## Step 9.2 local scaffold and connection checkpoint

The actual API binding is `DB`, as specified by step 9.2, plus private `TOOLS_BUCKET`. `KORTEX_DB` above was the provisional 9.0 name and remains confined to that test harness; no public contract changes. api/wrangler.jsonc has local emulated bindings and intentionally empty remote bindings, no account ID or real domains. Its compatibility date is 2026-10-26, supported by the pinned runtime; no AI binding. The authenticated OAuth session currently has Pages access but D1/R2 inventory reads return error 10000. Existing remote resources cannot be established with these permissions. The owner requested a stop at this point. See infra/cloudflare-setup.md for the exact status and reauthorization/resume procedure. No remote provisioning or deployment has occurred.

Checkpoint resolved on 2026-10-27: the owner activated the dedicated kortex profile, bound to this project. D1/R2 inventory reads now succeed with Workers/D1 scopes. Two existing account databases are left untouched; the default R2 inventory returns no buckets, and the specifically named staging/production Workers are not found. Remote bindings remain intentionally unconfigured until provisioning at 9.6. See docs/evidence/cloudflare-kortex-profile-9.2.json; no remote writes occurred.
