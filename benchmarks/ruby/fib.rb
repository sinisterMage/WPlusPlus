def fib(n)
  a = 0
  b = 1
  n.times do
    t = a + b
    a = b
    b = t
  end
  a
end

puts fib(45)

