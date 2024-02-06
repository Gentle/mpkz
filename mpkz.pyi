"""
this is meant as a replacement for json files on disk, the file format is optimized for fast reads
while still writing faster than python's json module while getting decent compression ratios.

mpkz is just messagepack with zstd compression, but implemented as efficient as possible.
Running some experiments, the default compression level of 8 is giving the best performance to compression ratio

messagepack can encode a superset of json, adding types for binary data and integers.
This means you can use mpkz as a drop-in replacement for json without any real downsides
"""

def load(fp):
    """
    load an mpkz from a File-Like Object
    """

def loads(bytes):
    """
    load an mpkz from a `bytes` instance
    """

def dump(obj, fp, *, level=8):
    """
    write a python object to a file in mpkz format.
    The default compression level of 8 is usually the best option
    """

def dumps(obj, *, level=8):
    """
    convert a pythob object to mpkz and return it as a `bytes` object .
    The default compression level of 8 is usually the best option
    """
