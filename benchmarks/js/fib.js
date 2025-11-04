let a = 0;
let b = 1;
for (let i = 0; i < 45; i++) {
  const t = a + b;
  a = b;
  b = t;
}
console.log(a);

