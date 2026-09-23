# Lineage of `official-rtd-skills`

This embedded bundle is now a status page and pointer to checked-in RTD sources. It does
not fetch external skill content and does not claim that a separate official RTD skills
repository exists. The historical source for the adjacent audit catalog is recorded in
`maintenance/UPSTREAMS.md`.

## Refresh protocol

If an RTD-owned skills repository is published and verified, update the embedded status
page with its actual repository and reviewed scope. Otherwise, keep the local source
pointers current when framework, CLI, or SDK package layouts change. Rebuild the RTD CLI
after editing the embedded page.
