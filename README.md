# mpkz - direct MessagePack zstd writer for Python

this is meant as a replacement for json files on disk, the file format is optimized for fast reads
while still writing faster than python's json module while getting decent compression ratios.

mpkz is just messagepack with zstd compression, but implemented as efficient as possible.
Running some experiments, the default compression level of 8 is giving the best performance to compression ratio

messagepack can encode a superset of json, adding types for binary data and integers.
This means you can use mpkz as a drop-in replacement for json without any real downsides

## Streaming

MessagePack was designed as a buffered protocol, there can be multiple messages in a single stream

In our case, this means that lists can be decoded on a per-line basis, potentially saving Memory (see example below)

for this reason, if the object is a list, it automatically gets encoded as a stream instead.

## Why not use messagepack and zstd from pypi?

with the python packages, you have to first encode the whole object into memory as MessagePack,
and then compress those bytes to zstd, and then write those compressed bytes to a file.

This quickly becomes impractical with larger amounts of data, so this implementation directly
serializes the python objects into a streaming zstd compressor, avoiding copying data more than once.

# API

## Basic

if you just want something that works like json, you can use the load/dump functions

```python
import mpkz

# Working with Files
with open("example.mpz", "wb") as f:
    mpkz.dump([1, 2, 3, 4, 5], f)
with open("example.mpz", "rb") as f:
    assert mpkz.load(f) == [1, 2, 3, 4, 5]

# Working with Bytes
input = { "greeting": "Hello World" }
binary = mpkz.dumps(input)
output = mpkz.loads(binary)
assert input == output
```

## Streaming

the Streaming API is useful for cases where the data you want to write to the file would not entirely fit into memory.

```python
import mpkz

# saving all rows of a Django Model into a file
writer = mpkz.create("export.mpz")
writer.extend(MyModel.objects)

# you can also append records one by one
writer = mpkz.create("example2.mpz")
writer.append("hello")
writer.append("world")

# and this is how you would iterate over the contents of a file without loading the whole file into memory
import mkpz
for row in mpkz.open("export.mpz"):
    print(row)
```
