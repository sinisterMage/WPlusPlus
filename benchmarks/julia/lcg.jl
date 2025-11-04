# Linear Congruential Generator benchmark (Int32 arithmetic)

function lcg(n::Int)
    a = Int32(1664525)
    c = Int32(1013904223)
    x = Int32(0)
    @inbounds for i in 1:n
        x = a * x + c
    end
    return x
end

const N = 50_000_000
println(lcg(N))

