const N = +(process.env.N || '50000000');
const a = 1664525|0;
const c = 1013904223|0;
let x = 0|0;
for (let i = 0; i < N; i++) {
  x = (Math.imul(a, x) + c) >>> 0;
}
console.log(x);

