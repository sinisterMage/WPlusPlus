# Branch-heavy loop: count numbers divisible by 3 in [0, N)

function count_div3(n::Int)
    c = 0
    @inbounds for i in 0:n-1
        # i % 3 == 0  <=>  i - (i ÷ 3) * 3 == 0
        q = i ÷ 3
        r = i - q * 3
        c += (r == 0)
    end
    return c
end

const N = 50_000_000
println(count_div3(N))

