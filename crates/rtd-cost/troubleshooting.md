# Troubleshooting

## Rtd Framework change

If Rtd framework code got updated, the expectations need to be changed. Follow these steps:

```bash
# required; can be omitted if cargo-insta is installed
$ cargo install cargo-insta

# run in ./rtd-cost
$ cargo insta test --review
```
