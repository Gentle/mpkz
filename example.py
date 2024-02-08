import sys
import time
import json
import mpkz
from timeit import timeit

def basic_bench(input_file, iterations):
    with open(input_file) as f:
        json_string = f.read()
    data = json.loads(json_string)
    mpz_bytes = mpkz.dumpb(data)

    duration = timeit(lambda: mpkz.loadb(mpz_bytes), number=iterations)
    print(f"loading mpkz, {iterations} iterations, average duration {duration / iterations}")

    duration = timeit(lambda: json.loads(json_string), number=iterations)
    print(f"loading json, {iterations} iterations, average duration {duration / iterations}")

if __name__ == "__main__":
    basic_bench(sys.argv[1], 10)
