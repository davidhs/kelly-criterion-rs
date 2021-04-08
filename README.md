# Kelly Criterion simulation

A simulation for the example given by the Wikipedia Kelly criterion article [^1].

![Result graph](cover.png "Result graph")

## Notes

Running:

```sh
cargo run 1000000
```

takes about 990 seconds, whereas running:

```sh
cargo run --release 1000000
```
takes about 9 seconds, which is roughly about 100x faster, on a AMD Ryzen 7 3700X 8-Core processor (16 threads).

## References

* [^1] <https://en.wikipedia.org/wiki/Kelly_criterion#Example>
