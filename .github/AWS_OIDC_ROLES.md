# RTD CI external infrastructure

The fork does not inherit the upstream operator's AWS account, buckets, roles,
Teleport cluster, release archives, or operations repository. These resources
must be provisioned and verified by the RTD operator before the associated CI
features are enabled. Repository variables below default to empty.

## AWS build cache

| Repository variable | Purpose |
| --- | --- |
| `RTD_AWS_ACCOUNT_ID` | RTD-owned 12-digit AWS account ID used by `allowed-account-ids` |
| `RTD_AWS_SCCACHE_RW_ROLE_ARN` | OIDC role for trusted branch/tag builds |
| `RTD_AWS_SCCACHE_RO_ROLE_ARN` | Optional OIDC role for same-repository pull requests |
| `RTD_SCCACHE_BUCKET` | RTD-owned S3 cache bucket |
| `RTD_AWS_REGION` | AWS region for both optional buckets and roles |

`setup-sccache` uses S3 only when a caller passes a role or static credentials.
Without credentials, Rust builds use the local runner cache. If credentials are
provided without account ID, bucket, and region, the action fails before contacting
AWS. Trusted-ref and same-repository PR gates in the callers remain in force.
The scheduled cache-warmup build is skipped until the RW role, account ID, bucket, and region are configured.

Before setting these variables, create the roles in the RTD-owned account with
an OIDC trust policy bound to the actual `github.repository`, the intended refs,
and the minimum S3 permissions. Do not copy an upstream account ID or trust
policy. No workflow currently passes static AWS access keys to `setup-sccache`.

## Release archives

| Repository variable | Purpose |
| --- | --- |
| `RTD_AWS_RELEASES_ROLE_ARN` | RTD-owned OIDC role for release archive uploads |
| `RTD_RELEASES_S3_BUCKET` | RTD-owned release archive bucket |
| `RTD_RELEASE_ARCHIVE_BASE_URL` | Optional HTTPS base URL for reading existing public archives |

The release S3 upload job is skipped unless its role, account ID, bucket, and region are
all configured. When enabled, upload errors fail the job. Without a configured
archive base URL, release builds compile their binaries instead of requesting
an inherited public S3 endpoint. GitHub release artifacts remain independent
of this optional S3 copy.

## Other external services

| Repository variable or secret | Default behavior |
| --- | --- |
| `RTD_OPERATIONS_REPOSITORY` and `DOCKER_BINARY_BUILDS_DISPATCH` | External repository dispatch steps are skipped when the repository variable is empty; a missing token fails an enabled dispatch. |
| `RTD_WALRUS_REPOSITORY` and `WALRUS_REPO_DISPATCH` | Walrus release dispatch is skipped without a verified repository. |
| `RTD_SIMTEST_TELEPORT_PROXY`, `RTD_SIMTEST_TELEPORT_JOIN_TOKEN`, and `RTD_SIMTEST_HOST` | Remote simulator runs only by manual dispatch and fails before network access when any value is missing. |
| `RTD_ENABLE_CHOCOLATEY_PUBLISH` and `CHOCO_API_KEY` | Chocolatey publication is off unless the variable is `true`; an enabled job fails if the API key is absent. |
| `RTD_ENABLE_PRETTIER_MOVE_PUBLISH` | npm publication is off unless the variable is `true`; dry runs remain available. |
| `RTD_PRETTIER_MOVE_NPM_PACKAGE` and `RTD_PRETTIER_MOVE_NPM_VERSION` | The published-package formatter check is skipped until both are configured. Local formatter tests still run. |
| `RTD_ENABLE_RELEASE_COMPATIBILITY_TESTS` | Scheduled tests against published RTD release binaries are skipped unless this is `true`. |

Configure external destinations only after verifying ownership and access.
Local Rust, Move, and documentation build steps do not require these services.
