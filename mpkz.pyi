"""
this is meant as a replacement for json files on disk, the file format is optimized for fast reads
while still writing faster than python's json module while getting decent compression ratios.

mpkz is just messagepack with zstd compression, but implemented as efficient as possible.
Running some experiments, the default compression level of 8 is giving the best performance to compression ratio

messagepack can encode a superset of json, adding types for binary data and integers.
This means you can use mpkz as a drop-in replacement for json without any real downsides
"""

from typing import Iterable, Iterator


def load(fp):
    """
    load an mpkz from a File-Like Object
    """

def loadb(data: bytes):
    """
    load an mpkz from a `bytes` instance
    """

def dump(obj, fp, *, level=8):
    """
    write a python object to a file in mpkz format.
    The default compression level of 8 is usually the best option
    """

def dumpb(obj, *, level=8):
    """
    convert a python object to mpkz and return it as a `bytes` object .
    The default compression level of 8 is usually the best option
    """

def open(filename) -> Iterable:
    """
    open an mpkz file that contains multiple records

    returns a memory-efficient Iterator over the records

    Example:
    ```
    for record in mpkz.open("stream.mpz"):
        print(f"hello {record["name"]}"")
    ```
    """

def openb(data: bytes):
    """
    open an mpkz buffer as a streaming iterator
    """

def create(filename, *, level=8) -> MpkzWriter:
    """
    create a new multi-record mpkz file

    because of the way zstd compression works,
    you can only create new files.
    Opening for appending is not possible

    Example:
    ```
    with mpkz.create("stream.mpz") as archive:
        archive.put_record({"name": "user"})
        archive.put_record({"name": "world"})
    ```
    """

class MpkzWriter:
    def append(self, obj):
        """
        write a record to the stream. A stream can contain any amount of records
        """

    def extend(self, iterable):
        """
        write all records from the iterable to the stream
        """

    def finish(self):
        """
        Finish the Archive. This will happen automatically
        when the writer is deleted or the context is exited
        """
