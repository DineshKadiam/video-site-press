<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Hyper 8 Docs

To generate the docs run this binary with a single argument,
specifying the folder to which the docs should be written.

**Take extra care specifying the target directory path, this directory gets
wiped and rewritten in the process.**

From the root of the hyper8 workspace run this command:

```bash
cargo run --package hyper8-docs -- /path/to/docs
```

## Check internal links/references

For automatically checking integrity of internal cross-links in the docs,
including references to anchors on pages, use a tool like
<https://linkchecker.github.io/linkchecker/index.html> (with the
`AnchorCheck` plugin enabled in
<https://linkchecker.github.io/linkchecker/man/linkcheckerrc.html>) on the
generated docs.
