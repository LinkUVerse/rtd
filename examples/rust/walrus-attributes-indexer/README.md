# Walrus Attributes Indexer

This is an extension of the [Custom Indexer guide](https://docs.rtd.io/guides/developer/advanced/custom-indexer) to show how to index Walrus blobs and their associated `Metadata` dynamic fields.

Walrus is a separate service. This example requires a Walrus deployment on
RTD, its `Metadata` type, and a checkpoint store for the same RTD network.
Set `RTD_WALRUS_METADATA_TYPE` to the deployed dynamic-field StructTag and
`RTD_CHECKPOINT_STORE_URL` to the actual checkpoint store before running.

## Quickstart

Given a service that allows users to upload blog posts to Walrus and creates an associated `Metadata` dynamic field with `view_count`, `title`, and `publisher` (Rtd address that created the Walrus blob), you can write a corresponding indexer that commits these attributes to a store of your choice to emulate a blog post platform. Then, users can:
- Upload blog posts with titles
- View their own posts and metrics
- Delete posts they created
- Edit post titles
- Browse posts by other publishers


To run the indexer:

```sh
RUST_LOG=info cargo run --release -- \
    --remote-store-url "$RTD_CHECKPOINT_STORE_URL" \
    --metadata-dynamic-field-type "$RTD_WALRUS_METADATA_TYPE"
```

Other useful commands:
```sh
# Get the status of a blob, such as its expiry epoch, when it was initially certified, etc.
walrus blob-status --blob-id {BLOB_ID}
# List all blobs for the current address, including expired ones.
walrus list-blobs --include-expired
# Set a path: value attribute pair on the Metadata dynamic field of a Blob object on Rtd.
walrus set-blob-attribute {Rtd blob object id} --attr "title" {title} --attr "view_count" {view_count}
```

```sh
# Creates a database and sets up the __diesel_schema_migrations table. Does not run any migrations.
diesel setup                                                                \
    --database-url=... \
    --migration-dir migrations
# Applies all pending migrations and updates the __diesel_schema_migrations table.
diesel migration run                                                        \
    --database-url=... \
    --migration-dir migrations
# Drops the entire database and recreates it from scratch by running all migrations from the beginning. Deletes all existing data.
diesel database reset --database-url=... --migration-dir migrations
```

## Blog Post Pipeline

The Blog Post pipeline is a sequential pipeline that writes the latest state of the `Metadata` dynamic fields to the `blog_post` table. It operates on a checkpoint granularity, and upserts records such that only the final update to an object in a checkpoint is persisted.

## Chain-agnostic Indexer

The StructTag of the `Metadata` dynamic field is passed to the service through
`--metadata-dynamic-field-type` or `RTD_WALRUS_METADATA_TYPE`.

## Defaults

As of writing, the SequentialConfig is defined [here](https://github.com/LinkUVerse/rtd/blob/main/crates/rtd-indexer-alt-framework/src/pipeline/sequential/mod.rs) and the committer config defaults are:
```
impl Default for CommitterConfig {
    fn default() -> Self {
        Self {
            write_concurrency: 5,
            collect_interval_ms: 500,
            watermark_interval_ms: 500,
        }
    }
}
```

The ingestion config is defined [here](https://github.com/LinkUVerse/rtd/blob/main/crates/rtd-indexer-alt-framework/src/ingestion/mod.rs) with defaults configured to:
```
impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            ingest_concurrency: ConcurrencyConfig::Adaptive { initial: 1, min: 1, max: 500, dead_band: None },
            retry_interval_ms: 200,
            // ...streaming fields elided
        }
    }
}
```

This means that by default, the blog post pipeline will have a write concurrency of 5, and the adaptive ingestion controller throttles fetch concurrency as the pipeline's subscriber channel fills.

## Follow-Along
After deploying Walrus on your RTD network, upload a blob and use the IDs
returned by that deployment. Then run the indexer through a checkpoint that
contains the metadata update:

```sh
walrus store src/handlers/blog_post.rs
walrus set-blob-attribute "$RTD_BLOB_OBJECT_ID" --attr view_count 5 --attr title "Blog post module" --attr publisher "$RTD_PUBLISHER_ADDRESS"
walrus get-blob-attribute "$RTD_BLOB_OBJECT_ID"
```

The row written by the indexer must reference the actual blob object and
publisher from that RTD network. No upstream checkpoint or object ID applies.
