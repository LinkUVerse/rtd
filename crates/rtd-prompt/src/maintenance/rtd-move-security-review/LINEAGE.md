# Lineage of the Move security review catalog

The `SM-*` rules were adapted from constructive Move guidance in the historical upstream
[`MystenLabs/skills`](https://github.com/MystenLabs/skills) repository at the commit recorded
in `crates/rtd-prompt/src/maintenance/UPSTREAMS.md`. A constructive rule can suggest an
audit failure mode when violated. This derivation does not establish that every upstream
rule applies unchanged to RTD.

Rule `Source:` annotations retain the paths used when the catalog was created. Those paths
refer to the historical upstream snapshot, even where their names were mechanically
renamed during the fork. Read them as lineage, then verify behavior in the current RTD
framework before reporting a finding. Rules tagged `[+domain]` come from broader Move
audit practice rather than a specific upstream skill.

## Refresh protocol

1. Review the pinned upstream snapshot in `maintenance/UPSTREAMS.md` and the proposed new
   upstream revision.
2. For each changed recommendation, inspect the current local RTD framework implementation.
3. Update affected `SM-*` rules and source annotations only where the behavior is supported.
4. Rebuild the RTD CLI, which embeds this catalog at build time.
