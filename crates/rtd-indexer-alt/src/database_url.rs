// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use anyhow::Result;
use anyhow::anyhow;
use anyhow::ensure;
use url::Url;

pub fn resolve(fallback: Url, path: Option<&Path>) -> Result<Url> {
    let Some(path) = path else {
        return Ok(fallback);
    };
    ensure!(path.is_absolute(), "database URL file must be absolute");
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| anyhow!("database URL file unavailable"))?;
    let before = file
        .metadata()
        .map_err(|_| anyhow!("database URL file metadata unavailable"))?;
    // SAFETY: geteuid has no arguments or memory safety preconditions.
    let uid = unsafe { libc::geteuid() };
    ensure!(
        before.is_file()
            && before.uid() == uid
            && before.mode() & 0o077 == 0
            && before.len() <= 4096,
        "database URL file must be an owned private regular file <=4096 bytes"
    );
    let mut bytes = Vec::new();
    (&mut file)
        .take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| anyhow!("database URL file unreadable"))?;
    let after = file
        .metadata()
        .map_err(|_| anyhow!("database URL file metadata unavailable"))?;
    ensure!(
        bytes.len() as u64 == before.len()
            && before.len() == after.len()
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec(),
        "database URL file changed during read"
    );
    let text =
        std::str::from_utf8(&bytes).map_err(|_| anyhow!("database URL file is not UTF-8"))?;
    let text = text.strip_suffix('\n').unwrap_or(text);
    ensure!(
        !text.is_empty() && !text.bytes().any(|c| c.is_ascii_whitespace() || c == 0),
        "database URL file must contain one URL"
    );
    let url = Url::parse(text).map_err(|_| anyhow!("invalid database URL file"))?;
    ensure!(
        matches!(url.scheme(), "postgres" | "postgresql")
            && url.host_str().is_some()
            && url.fragment().is_none(),
        "invalid database URL file"
    );
    Ok(url)
}
