# rtd prompt — expert Rtd and Move knowledge for AI agents

`rtd prompt` prints expert Rtd and Move knowledge from embedded skill bundles,
organized into **categories**.

## How to use this

Read the available categories (`rtd prompt categories`), try to match one to the
task, then drill into its bundles. Each skill is a two-tier bundle: `SKILL.md`
routes, reference files hold content. **Read every reference file** before
applying — `--all` loads them in one call.

```sh
rtd prompt categories                    # see the available categories
rtd prompt category <name> --all         # read every bundle's content in one call
rtd prompt category <name>               # read a category's workflow + skill list
rtd prompt category <name> --list        # list bundle and reference file names and sizes (no content)
```

Skills can also be reached directly:

```sh
rtd prompt skills                        # list all skill bundles, flat
rtd prompt skill <bundle> --all          # read SKILL.md + every reference file
rtd prompt skill <bundle>                # read a bundle's SKILL.md
rtd prompt skill <bundle> --list         # list reference file names and sizes (no content)
rtd prompt skill <bundle> --file <ref>   # read a specific reference file
```