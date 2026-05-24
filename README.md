# muscle-rs

*This crate is called muscle on crate.io due to a clumpsy initial upload. Unfortunately crate.io does not allow old creates to be completely removed. To avoid polluting the namespace further,
the crate name has been kept, and this author has learned to check 5 times before upload (and then one more time). Apologizes for any confusion!*

A Rust implementation of [MUSCLE](https://drive5.com/muscle5) (MUltiple Sequence Comparison by Log-Expectation), a widely-used multiple sequence alignment tool for biological sequences.

* 2026-05-24: Careful testing on real data now possible. But beware, there is global state that should be removed in future versions!

## This is an LLM-mediated faithful (hopefully) translation, not the original code! 

Most users should probably first see if the existing original code works for them, unless they have reason otherwise. The original source
may have newer features and it has had more love in terms of fixing bugs. In fact, we aim to replicate bugs if they are present, for the
sake of reproducibility! (but then we might have added a few more in the process)

There are however cases when you might prefer this Rust version. We generally agree with [this manifesto](https://rewrites.bio/) but more specifically:
* We have had many issues with ensuring that our software works using existing containers (Docker, PodMan, Singularity). One size does not fit all and it eats our resources trying to keep up with every way of delivering software
* Common package managers do not work well. It was great when we had a few Linux distributions with stable procedures, but now there are just too many ecosystems (Homebrew, Conda). Conda has an NP-complete resolver which does not scale. Homebrew is only so-stable. And our dependencies in Python still break. These can no longer be considered professional serious options. Meanwhile, Cargo enables multiple versions of packages to be available, even within the same program(!)
* The future is the web. We deploy software in the web browser, and until now that has meant Javascript. This is a language where even the == operator is broken. Typescript is one step up, but a game changer is the ability to compile Rust code into webassembly, enabling performance and sharing of code with the backend. Translating code to Rust enables new ways of deployment and running code in the browser has especial benefits for science - researchers do not have deep pockets to run servers, so pushing compute to the user enables deployment that otherwise would be impossible
* Old CLI-based utilities are bad for the environment(!). A large amount of compute resources are spent creating and communicating via small files, which we can bypass by using code as libraries. Even better, we can avoid frequent reloading of databases by hoisting this stage, with up to 100x speedups in some cases. Less compute means faster compute and less electricity wasted
* LLM-mediated translations may actually be safer to use than the original code. This article shows that [running the same code on different operating systems can give somewhat different answers](https://doi.org/10.1038/nbt.3820). This is a gap that Rust+Cargo can reduce. Typesafe interfaces also reduce coding mistakes and error handling, as opposed to typical command-line scripting

But:

* **This approach should still be considered experimental**. The LLM technology is immature and has sharp corners. But there are opportunities to reap, and the genie is not going back into the bottle. This translation is as much aimed to learn how to improve the technology and get feedback on the results.
* Translations are not endorsed by the original authors unless otherwise noted. **Do not send bug reports to the original developers**. Use our Github issues page instead.
* **Do not trust the benchmarks on this page**. They are used to help evaluate the translation. If you want improved performance, you generally have to use this code as a library, and use the additional tricks it offers. We generally accept performance losses in order to reduce our dependency issues
* **Check the original Github pages for information about the package**. This README is kept sparse on purpose. It is not meant to be the primary source of information
* **If you are the author of the original code and wish to move to Rust, you can obtain ownership of this repository and crate**. Until then, our commitment is to offer an as-faithful-as-possible translation of a snapshot of your code. If we find serious bugs, we will report them to you. Otherwise we will just replicate them, to ensure comparability across studies that claim to use package XYZ v.666. Think of this like a fancy Ubuntu .deb-package of your software - that is how we treat it

This blurb might be out of date. Go to [this page](https://github.com/henriksson-lab/rustification) for the latest information and further information about how we approach translation

## Build

```sh
cargo build --release
```

The default build is library-only. Build the optional CLI binary with:

```sh
cargo build --release --features cli-bin
```

The release binary is then written to `target/release/muscle_rs` on Unix-like
systems or `target/release/muscle_rs.exe` on Windows.

## CLI Usage

The binary is named `muscle_rs`, but the Clap command is configured as
`muscle` for compatibility with the original option style.

Examples:

```sh
target/release/muscle_rs -msastats muscle/test_data/fa/BB11001
target/release/muscle_rs -make_a2m muscle/test_data/fa/BB11001 -output out.a2m
target/release/muscle_rs -trimtoref input.fa -output trimmed.fa
target/release/muscle_rs -tree_subset_nodes tree.nwk -nodes nodes.tsv -output subset.nwk
target/release/muscle_rs -divide_tree tree.nwk -label1 A -label2 B -subtreeout sub.nwk -supertreeout super.nwk
```

Clap also accepts long double-dash forms and hyphen aliases for many options:

```sh
target/release/muscle_rs --strip-gappy-cols input.fa --output stripped.fa
```

On Windows, use `target\release\muscle_rs.exe` instead.

## Library Usage

The high-level API uses a builder pattern and runs the translated MUSCLE core
in-process:

```rust
use muscle_rs::Muscle;

let aligned = Muscle::builder()
    .input_fasta(">a\nACDEFG\n>b\nACDFG\n")
    .threads(5)
    .build()?
    .into_fasta();
```

This currently exposes the normal `-align` path. Calls are serialized inside the
API because the faithful translation still uses C++-style process-global state
for options such as thread count, alphabet selection, HMM parameters and progress
flags.

Removing that global state is required future work for a fully idiomatic and
concurrently usable Rust library API. The current serialization is a
compatibility workaround, not the intended long-term design.

Library callers that need exact raw CLI behavior can call the dispatcher
directly without enabling or spawning the binary:

```rust
muscle_rs::app::main_with_args(vec![
    "muscle".into(),
    "-align".into(),
    "input.fa".into(),
    "-output".into(),
    "out.afa".into(),
    "-threads".into(),
    "5".into(),
]);
```

## Testing

Run the standard Rust checks:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
```

The exact-output parity tests compare translated Rust behavior against an
in-tree build of the original MUSCLE C++ binary. The command below is for a
Unix-like shell with OpenMP-capable `g++`; on other platforms, set
`MUSCLE_CPP_BIN` to an existing original MUSCLE binary or let those tests skip.
Build it once before running the test suite:

```sh
cd muscle/src
echo '"0"' > gitver.txt
g++ -O2 -fopenmp -o muscle $(grep -oE 'ClCompile Include="[^"]+\.cpp"' muscle.vcxproj | sort -u | sed 's/.*"\(.*\)"/\1/')
```

The tests auto-discover the binary at `muscle/src/muscle` relative to the
crate root. Set `MUSCLE_CPP_BIN=/path/to/muscle` to override. When neither is
available, parity tests print a skip message and exit successfully.

The tests also exercise real upstream fixtures in `muscle/test_data`.

## Benchmarks

These are single-run local measurements using `/usr/bin/time -v` wall seconds
and maximum resident set size. They are translation sanity checks, not stable
published benchmarks. The C++ binary is `muscle/src/muscle`; the Rust binary is
`target/release/muscle_rs`.

The BAliBASE rows use fixtures from `muscle/test_data/fa`. C++ and Rust outputs
were byte-identical except for `-super6 BB11005 -threads 5`, which is
thread-order sensitive in the original clustering path. Ratio columns are
`C++ / Rust`, so values above `1.00x` mean Rust is faster or uses less RSS.

| Command | Fixture | Threads | C++ time | Rust time | Speed ratio | C++ RSS | Rust RSS | RSS ratio | Same output |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `-align` | BB11005 | 5 | 1.38s | 0.98s | 1.41x | 104.5 MiB | 48.8 MiB | 2.14x | yes |
| `-align` | BB11007 | 5 | 0.60s | 0.50s | 1.20x | 64.9 MiB | 44.4 MiB | 1.46x | yes |
| `-super5` | BB11005 | 1 | 9.45s | 9.71s | 0.97x | 36.2 MiB | 21.4 MiB | 1.69x | yes |
| `-super5` | BB11005 | 5 | 4.97s | 9.70s | 0.51x | 128.8 MiB | 36.0 MiB | 3.58x | yes |
| `-super5` | BB11007 | 1 | 4.27s | 4.34s | 0.98x | 25.1 MiB | 17.0 MiB | 1.48x | yes |
| `-super5` | BB11007 | 5 | 2.47s | 4.63s | 0.53x | 78.8 MiB | 37.7 MiB | 2.09x | yes |
| `-super6` | BB11005 | 1 | 3.73s | 3.61s | 1.03x | 29.2 MiB | 19.9 MiB | 1.47x | yes |
| `-super6` | BB11005 | 5 | 1.36s | 3.23s | 0.42x | 107.7 MiB | 61.5 MiB | 1.75x | no |
| `-super6` | BB11007 | 1 | 1.52s | 1.49s | 1.02x | 21.8 MiB | 17.4 MiB | 1.25x | yes |
| `-super6` | BB11007 | 5 | 0.58s | 1.07s | 0.54x | 62.9 MiB | 45.3 MiB | 1.39x | yes |

The RDRP fixture is a deterministic subset containing the first 25 sequences
from `muscle/test_data/rdrp/rdrp.fa` (`25` sequences, `10059` residues, max
length `471`, average length `402.36`). C++ and Rust outputs were
byte-identical.

| Command | Fixture | Threads | C++ time | Rust time | Speed ratio | C++ RSS | Rust RSS | RSS ratio |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `-align` | RDRP25 | 1 | 19.13s | 11.13s | 1.72x | 43.0 MiB | 24.0 MiB | 1.79x |
| `-align` | RDRP25 | 5 | 5.86s | 2.96s | 1.98x | 109.9 MiB | 53.4 MiB | 2.06x |


## Faithfulness note: unstable quicksort

The original MUSCLE C++ source uses a custom **unstable** Hoare-partition
quicksort (`QuickSortOrder` / `QuickSortOrderDesc` in `muscle/src/sort.h`) in
several hot paths — uclust, uclustpd, uclustpd2, eesort, sweeper, treesplitter,
length-ordering inside `MultiSequence::GetLengthOrder`, the `USorter` top-hit
search, and a few internal selection routines. Because the sort is unstable,
**elements with equal keys come out in an order determined by the partition
pivot, not by their input order**.

To preserve byte-for-byte parity with the C++ binary, this Rust port replays
the same Hoare partition (see `quick_sort_order_desc_by` / `quick_sort_order_by`
in `src/countsort.rs`) at the same call sites instead of using Rust's stable
`sort_by`. A stable sort would *also* be a deterministic ordering — just a
different one — which is enough to silently change cluster centroids, output
sequence ordering, and downstream computation when ties are present.

**When this can affect reproducibility:**

- **Across MUSCLE versions / builds** — the C++ quicksort's tie-break depends
  on pivot choice (here, the midpoint), so even small changes to that algorithm
  (different compiler optimisations, vectorisation, parallel sort variants) can
  reshuffle tied keys.
- **uclust / uclustpd / uclustpd2 on inputs with tied sequence lengths** —
  iteration order through `GetLengthOrder` determines which sequences become
  centroids first; ties make this order pivot-dependent.
- **`USorter` and `eesort` when several DB sequences share the top word-count
  or expected-accuracy score** — the reported "best hit" / output ordering
  depends on the unstable sort.
- **`Sweeper` parameter listings, `TreeSplitter` size ordering, and any
  benchmark CSV that ranks ties (`cmd_newbench_selectpfams`)**.

**When this is unlikely to matter:**

- Inputs whose sorted keys are all distinct (most "real" protein/DNA data with
  varied sequence lengths, distinct E-values, distinct PD scores).
- Commands that don't internally rank by a tied numerical key — `derep`,
  `make_a2m`, `squeeze_inserts`, `trimtoref`, `strip_gappy*`, basic stats, etc.
  These match the C++ binary regardless of sort behavior.
- Downstream consumers that read the output set rather than the output
  *ordering* (e.g. "the set of cluster centroids" is unaffected; "the order
  centroids are printed in" can flip).

If you care about reproducibility across MUSCLE versions or unrelated runs,
**don't rely on the order of tied elements** in MUSCLE's output — explicitly
re-sort with a stable key downstream. Parity is preserved here as a faithful
translation property, not a guarantee that the chosen order is canonical.


## Citing

Edgar RC., Muscle5: High-accuracy alignment ensembles enable unbiased assessments of sequence homology and phylogeny. Nature Communications 13.1 (2022): 6968.
https://www.nature.com/articles/s41467-022-34630-w.pdf

Edgar RC. and Tolstoy I., Muscle-3D: scalable multiple protein structure alignment (2024) BioRxiv.

## License

GPL-3.0
