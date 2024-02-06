# mpkz - direct MessagePack zstd writer for Python

this is meant as a replacement for json files on disk, the file format is optimized for fast reads
while still writing faster than python's json module while getting decent compression ratios.

mpkz is just messagepack with zstd compression, but implemented as efficient as possible.
Running some experiments, the default compression level of 8 is giving the best performance to compression ratio

messagepack can encode a superset of json, adding types for binary data and integers.
This means you can use mpkz as a drop-in replacement for json without any real downsides

## Why not use messagepack and zstd from pypi?

with the python packages, you have to first encode the whole object into memory as MessagePack,
and then compress those bytes to zstd, and then write those compressed bytes to a file.

This implementation directly serializes the python objects into a streaming zstd compressor,
avoiding copying data more than once
