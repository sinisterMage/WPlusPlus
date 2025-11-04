def lcg(n)
  a = 1664525
  c = 1013904223
  mask = 0xFFFFFFFF
  x = 0
  n.times do
    x = (a * x + c) & mask
  end
  x
end

n = (ENV['N'] || '50000000').to_i
puts lcg(n)
