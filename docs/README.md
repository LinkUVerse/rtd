# The Rtd documentation has moved

The pages behind [docs.rtd.io](https://docs.rtd.io) are no longer in this
repository. They now live in
**[LinkUVerse/rtd-docs](https://github.com/LinkUVerse/rtd-docs)**, along with the
Docusaurus site that renders them. The site itself is unchanged: same URLs, same
content.

| What you want | Where it is now |
| --- | --- |
| A documentation page | [`content/rtd/`](https://github.com/LinkUVerse/rtd-docs/tree/main/content/rtd) |
| The site, its theme and plugins | [`sites/rtd/`](https://github.com/LinkUVerse/rtd-docs/tree/main/sites/rtd) |
| To report a problem with the docs | [Open an issue](https://github.com/LinkUVerse/rtd-docs/issues/new) on `rtd-docs` |
| To fix a page yourself | The **Edit this page** link at the bottom of any page on docs.rtd.io, or a pull request against `rtd-docs` |

## What is still here, and why

Parts of the documentation are generated at build time from source in this
repository, and `rtd-docs` fetches them on every build:

| Path | Generates |
| --- | --- |
| `crates/rtd-framework/docs/` | the Move framework reference |
| `crates/`, `examples/` | the code embedded in pages by `ImportContent` |
| `release-notes/` | the release notes page |

Two things follow.

Comments marked `docs::#name` in Rust and Move source are **snippet anchors**, not
ordinary comments. A page quotes the lines between them, so renaming or removing
one breaks whatever page embeds it.

Moving or renaming any of the paths above breaks the documentation build in the
other repository, and nothing here will tell you. If you need to move one,
change
[`sources.json`](https://github.com/LinkUVerse/rtd-docs/blob/main/sources.json)
in `rtd-docs` in the same change.

The awesome-rtd lists used to be vendored here as a git subtree under
`docs/subtree`. `rtd-docs` now fetches them from
[rtd-foundation/awesome-rtd](https://github.com/rtd-foundation/awesome-rtd) and
[becky-rtd/awesome-rtd-gaming](https://github.com/becky-rtd/awesome-rtd-gaming)
directly, so there is nothing to keep in sync here.

## Why it moved

The machinery underneath seven documentation sites had been copied rather than
shared: the same fetcher written three times, the same frontmatter schema in
three repositories, shared components kept aligned across four repositories by a
sync bot. `rtd-docs` gives that one home.
