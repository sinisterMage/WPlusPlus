const N = +(process.env.N || '50000000');
let count = 0;
for (let i = 0; i < N; i++) {
  const q = (i / 3) | 0;
  const r = i - q * 3;
  if (r === 0) count++;
}
console.log(count);

