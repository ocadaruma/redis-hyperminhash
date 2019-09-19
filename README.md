# redis-hyperminhash

[![Build Status](https://travis-ci.org/ocadaruma/redis-hyperminhash.svg?branch=master)](https://travis-ci.org/ocadaruma/redis-hyperminhash)

A Redis module provides HyperLogLog and MinHash feature at once using [HyperMinHash](https://arxiv.org/abs/1710.08436) sketch.

redis-hyperminhash is written in Rust.

Features:

- Cardinality estimation
  - Same accuracy as of Redis built-in HLL (PFCOUNT)
- Similarity estimation
  - Estimate Jaccard index using MinHash
- Intersection cardinality estimation
  - By combining Jaccard index and union cardinality

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

Estimates Jaccard index between multiple sketches.

```
redis-cli> MH.SIMILARITY key other-key
"0.59999994040939497"
```

### MH.INTERSECTION

Estimates intersection cardinality between multiple sketches.

```
redis-cli> MH.INTERSECTION key other-key
(integer) 3
```

## Memory usage

Sketch size is 32KB per key.

Unlike Redis built-in HLL, redis-hyperminhash does not support sparse encoding now.

## Performance

HLL operations (MH.ADD, MH.COUNT, MH.MERGE) perform almost as fast as built-in HLL.

MH.SIMILARITY, MH.INTERSECTION are slightly slow. (2 or 3 times slower than HLL operations)

See results in [rough benchmark](benchmark/README.md).

## `MH.COUNT` Accuracy

`MH.COUNT` relies on [New cardinality estimation algorithms for HyperLogLog sketches](https://arxiv.org/abs/1702.01284), which is adopted in Redis built-in HLL.

Histogram of 500 experiments (true cardinality = 10000)

```
============== HyperMinHash ==============
09816- : **
09835- : **
09854- : ***
09873- : *********
09892- : ************************
09911- : ************************
09930- : *************************************
09950- : ****************************************************
09969- : ********************************************************************
09988- : ***************************************************************************
10007- : ******************************************************
10026- : *************************************************
10045- : ********************************
10064- : ************************************
10084- : ****************
10103- : *********
10122- : ***
10141- : **
10160- : **
10179- :
10199- : *
============== built-in HyperLogLog ==============
09797- : *
09817- : *
09837- :
09858- : ****
09878- : *************
09899- : **********************
09919- : ********************************************
09939- : **********************************************
09960- : ******************************************************
09980- : ********************************************************
10001- : *******************************************************************
10021- : ************************************************************
10041- : ***************************************
10062- : *************************************************
10082- : ***************
10103- : ***************
10123- : ********
10143- : **
10164- : **
10184- : *
10205- : *
```
