---
name: rtd-and-move-tools
description: >
  Use to get bytecode for a deployed Rtd package and produce a disassembled working view.
  One GraphQL call fetches every module's raw bytecode bytes; `rtd move disassemble`
  (already on the system, running `rtd prompt`) produces `.asm` files for analysis.
  Trigger on "fetch this package's bytecode", "get me the .mv for package X",
  "disassemble this package", or "I need to read a deployed Rtd package".
---

# Rtd and Move Tools

Get a deployed Rtd package's bytecode and produce `.mv` and `.asm` (disassembly) files.
One Rtd GraphQL call returns raw bytes for every module; `rtd move disassemble` produces
the working view module-by-module.

For what the disassembly conveys (and what's lost in compilation), see
`move-bytecode-comprehension`. The end-to-end procedure is in `fetch-and-disassemble.md`.
