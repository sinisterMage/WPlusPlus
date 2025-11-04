def fib(n: int) -> int:
    a = 0
    b = 1
    for _ in range(n):
        t = a + b
        a = b
        b = t
    return a

print(fib(45))

