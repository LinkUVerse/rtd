See [DVX-953](https://linear.app/linku-labs/issue/DVX-953/conflicts-between-same-versions-of-deps)


Test setup:

```
A ---> B ---> C
```

B has a Move.lock, but A and C don't.

DVX-953 meant that in this case, we would get the following errors:

```
Failed to build Move modules: When resolving dependencies for package A, conflicting dependencies found:
At C
        Bridge = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/bridge" }
        D = { local = "../D" }
        MoveStdlib = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/move-stdlib" }
        Rtd = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/rtd-framework" }
        RtdSystem = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/rtd-system" }
At B -> C
        Bridge = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/bridge" }
        D = { local = "../D" }
        MoveStdlib = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/move-stdlib" }
        Rtd = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/rtd-framework" }
        RtdSystem = { git = "https://github.com/LinkUVerse/rtd.git", rev = "04f11afaf5e0", subdir = "crates/rtd-framework/packages/rtd-system" }.
```
