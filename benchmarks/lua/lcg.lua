local N = tonumber(os.getenv("N") or "50000000")
local a = 1664525
local c = 1013904223
local MOD = 4294967296 -- 2^32
local x = 0

for _ = 1, N do
  x = (a * x + c) % MOD
end

print(x)

