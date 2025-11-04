local n = 45
local a = 0
local b = 1
for _ = 1, n do
  local t = a + b
  a = b
  b = t
end
print(a)

