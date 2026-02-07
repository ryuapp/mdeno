Deno.test("basic Promise", async () => {
  const p1 = new Promise((resolve) => {
    resolve("Promise resolved!");
  });
  const result = await p1;
  if (result !== "Promise resolved!") {
    throw new Error(`Expected "Promise resolved!", got "${result}"`);
  }
});

Deno.test("Promise chain", async () => {
  const result = await Promise.resolve(1)
    .then((x) => x + 1)
    .then((x) => x * 2);

  if (result !== 4) {
    throw new Error(`Expected 4, got ${result}`);
  }
});

Deno.test("async/await", async () => {
  async function testAsync() {
    const result = await Promise.resolve("async works!");
    return result;
  }

  const result = await testAsync();
  if (result !== "async works!") {
    throw new Error(`Expected "async works!", got "${result}"`);
  }
});

Deno.test("Promise.all", async () => {
  const values = await Promise.all([
    Promise.resolve(1),
    Promise.resolve(2),
    Promise.resolve(3),
  ]);

  if (values.length !== 3) {
    throw new Error(`Expected 3 values, got ${values.length}`);
  }
  if (values[0] !== 1 || values[1] !== 2 || values[2] !== 3) {
    throw new Error(`Expected [1, 2, 3], got [${values}]`);
  }
});
