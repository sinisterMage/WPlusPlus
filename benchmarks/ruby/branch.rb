def count_div3(n)
  c = 0
  i = 0
  while i < n
    q = i / 3
    r = i - q * 3
    c += 1 if r == 0
    i += 1
  end
  c
end

n = (ENV['N'] || '50000000').to_i
puts count_div3(n)

