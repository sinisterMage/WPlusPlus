local N = tonumber(os.getenv("N") or "50000000")
local count = 0

for i = 0, N - 1 do
  local q = math.floor(i / 3)
  local r = i - q * 3
  if r == 0 then
    count = count + 1
  end
end

print(count)

