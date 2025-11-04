import os

def count_div3(n: int) -> int:
    c = 0
    for i in range(n):
        q = i // 3
        r = i - q * 3
        if r == 0:
            c += 1
    return c

N = int(os.environ.get("N", "50000000"))
print(count_div3(N))

