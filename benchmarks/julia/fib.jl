# Iterative Fibonacci(n) with n=45 (fits in Int32)

function fib(n::Int)
    a::Int32 = 0
    b::Int32 = 1
    @inbounds for _ in 1:n
        t = a + b
        a = b
        b = t
    end
    return a
end

println(fib(45))

