# Environments and ownership

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
