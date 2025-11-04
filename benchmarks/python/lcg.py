import os

def lcg(n: int) -> int:
    a = 1664525
    c = 1013904223
    mask = 0xFFFFFFFF  # emulate 32-bit wrap-around
    x = 0
    for _ in range(n):
        x = (a * x + c) & mask
    return x

N = int(os.environ.get("N", "50000000"))
print(lcg(N))
