# Third-party components

tddlog is Apache-2.0 (LICENSE). NOTICE lists the third-party programs and
data the repository holds. This file records the third-party components
compiled into tddlog's binaries: the command line, the Python wheels, the
C library and the WebAssembly module. The BSD-2-Clause and MIT notices
they require are reproduced below.

## Rust crates

- [tididi](https://github.com/Tractables/tididi), the TDD library, and
  [goatd](https://github.com/Tractables/goatd), the tree-decomposition
  library: Apache-2.0.
- `itoa`, `libc`, `num-bigint`, `num-integer`, `num-rational`,
  `num-traits`, `rustc-hash` and `smallvec`; in the Python wheels `pyo3`,
  `pyo3-ffi` and `once_cell`; in the command line `tikv-jemallocator` and
  `tikv-jemalloc-sys`. Each is dual-licensed `MIT OR Apache-2.0` and is
  used here under Apache-2.0.

## C and C++ components

goatd's build script compiles its vendored C++ tree-decomposition solvers
and links them in. goatd's `docs/THIRD-PARTY.md` lists the files and the
changes goatd made to them; paths are relative to its
`vendor/treedecomp/upstream/`.

| Files | Upstream | License | Copyright |
|---|---|---|---|
| `flow-cutter-pace17/**`, `IFlowCutter.{cpp,hpp}` | [FlowCutter, PACE 2017](https://github.com/kit-algo/flow-cutter-pace17) | BSD-2-Clause | © 2016 Ben Strasser |
| `graph.{cpp,hpp}`, `bitset.hpp`, `utils.hpp` | [sharpSAT-TD](https://github.com/Laakeri/sharpsat-td) | MIT | © 2021 Tuukka Korhonen and Matti Järvisalo |
| `TreeDecomposition.{cpp,hpp}` | [treedecomp](https://github.com/meelgroup/treedecomp) | MIT | © 2023 Kenji Hashimoto |
| `treedecomp_defs.hpp` | [treedecomp](https://github.com/meelgroup/treedecomp) | MIT | © 2023 Authors of treedecomp |
| `time_mem.hpp` | [MiniSat](https://github.com/niklasso/minisat) / [CryptoMiniSat](https://github.com/msoos/cryptominisat) | MIT | © 2003–2006 Niklas Eén, Niklas Sörensson; © 2009–2020 the CryptoMiniSat authors |

The command line links [jemalloc](https://github.com/jemalloc/jemalloc), its
allocator, through `tikv-jemalloc-sys`: BSD-2-Clause, © 2002–present Jason
Evans, © 2007–2012 Mozilla Foundation, © 2009–present Facebook, Inc. The
Python wheels and the C library do not link it.

The WebAssembly module the site serves (`bindings/wasm`) is linked by
Emscripten with its system libraries: musl's C library (MIT, © 2005–2020
Rich Felker, et al.) and LLVM's libc++, libc++abi and compiler-rt
(Apache-2.0 with LLVM exceptions, which waive attribution for the parts
compiled into a program). It does not link jemalloc.

---

## BSD-2-Clause — FlowCutter

```
Copyright (c) 2016, Ben Strasser
All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

Redistributions of source code must retain the above copyright notice, this list
of conditions and the following disclaimer.
Redistributions in binary form must reproduce the above copyright notice, this
list of conditions and the following disclaimer in the documentation and/or
other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

## BSD-2-Clause — jemalloc

```
Copyright (C) 2002-present Jason Evans <jasone@canonware.com>.
All rights reserved.
Copyright (C) 2007-2012 Mozilla Foundation.  All rights reserved.
Copyright (C) 2009-present Facebook, Inc.  All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
1. Redistributions of source code must retain the above copyright notice(s),
   this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice(s),
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDER(S) ``AS IS'' AND ANY EXPRESS
OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.  IN NO
EVENT SHALL THE COPYRIGHT HOLDER(S) BE LIABLE FOR ANY DIRECT, INDIRECT,
INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE
OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

## MIT

Applies to the MIT-licensed components above, each under its own copyright line
as listed:

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```
