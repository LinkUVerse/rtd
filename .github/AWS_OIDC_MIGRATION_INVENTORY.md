# Fork CI infrastructure migration status

This file records the fork's current CI boundary. The original Sui IAM account,
S3 buckets, Teleport proxy, operations repository, and publication endpoints are
not RTD infrastructure. Their address values have been removed from executable
workflows. See [AWS_OIDC_ROLES.md](AWS_OIDC_ROLES.md) for the repository variables,
secrets, and safe defaults used now.

- All `setup-sccache` callers pass the RTD account ID, bucket, and region from repository
  variables. A missing role means a local build; a configured role without its
  account ID, bucket, or region fails validation.
- The release archive read path is optional. Release S3 upload requires an
  explicit RTD role, account ID, bucket, and region, and reports upload failures.
- Repository dispatches require an explicit operations or Walrus repository.
  The remote simulator is manual-only and needs a configured proxy, token, and host.
- Chocolatey and npm publication need explicit enablement. Published-package
  formatter and historical release tests stay disabled until a verified RTD
  package or release line is available.

No GitHub-hosted workflow, AWS role, bucket, package registry ownership, or
remote simulator was validated as part of the local fork. Validate each before
turning on its variable or secret.
