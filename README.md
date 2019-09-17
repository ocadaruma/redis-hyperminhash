# redis-hyperminhash

A Redis module provides HyperLogLog and MinHash feature at once based on [HyperMinHash](https://arxiv.org/abs/1710.08436).

redis-hyperminhash is written in Rust.

Features:

- Cardinality estimation
  - Same accuracy as of Redis built-in HyperLogLog (PFCOUNT)
- Similarity estimation
  - Estimate jaccard index by MinHash
- Intersection cardinality estimation
  - By combining jaccard index and union cardinality

## Installation

1. Download and extract binary from [Releases](https://github.com/ocadaruma/redis-hyperminhash/releases).
2. Load module.

```
redis-cli> MODULE LOAD /path/to/libredis_hyperminhash.so
```

### Build

You can build manually if necessary.

```
$ git clone https://github.com/ocadaruma/redis-hyperminhash.git
$ cd redis-hyperminhash
$ cargo build --release
$ cp target/release/libredis_hyperminhash.so /path/to/modules/
```

## Usage

### MH.ADD

```
redis-cli> MH.ADD key id1 id2 id3
(integer) 1
```

Same usage as `PFADD`.

### MH.COUNT

```
redis-cli> MH.COUNT key
(integer) 3
```

Same usage as `PFCOUNT`.

### MH.MERGE

```
redis-cli> MH.ADD other-key id1 id2 id3 id4 id5
(integer) 1
redis-cli> MH.MERGE dest key other-key
OK
redis-cli> MH.COUNT dest
(integer) 5
```

Same usage as `PFMERGE`.

### MH.SIMILARITY

```
redis-cli> MH.SIMILARITY key other-key
"0.59999994040939497"
```

### MH.INTERSECTION

```
redis-cli> MH.INTERSECTION key other-key
(integer) 3
```
