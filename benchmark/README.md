## Benchmark

Rough benchmark using ruby and 'redis' gem.

Environment: iMac 2019, 3.6GHz Corei9, 16GB DDR4

```
$ cd benchmark
$ bundle install --path=vendor/bundle 
$ bundle exec ruby throughput.rb
MH.ADD
Took total of: 3.717445 s
Per iteration: 3.717445e-05 s (0.03717445 ms)

PFADD
Took total of: 3.76532 s
Per iteration: 3.76532e-05 s (0.0376532 ms)

MH.COUNT
Took total of: 5.969141 s
Per iteration: 5.9691409999999996e-05 s (0.05969140999999999 ms)

PFCOUNT
Took total of: 3.808235 s
Per iteration: 3.8082349999999996e-05 s (0.038082349999999994 ms)

MH.MERGE
Took total of: 5.194098 s
Per iteration: 5.194098e-05 s (0.051940980000000005 ms)

PFMERGE
Took total of: 4.507003 s
Per iteration: 4.507003e-05 s (0.045070030000000004 ms)

MH.SIMILARITY
Took total of: 9.951946 s
Per iteration: 9.951946e-05 s (0.09951946 ms)

MH.INTERSECTION
Took total of: 11.926736 s
Per iteration: 0.00011926736 s (0.11926736 ms)
```
