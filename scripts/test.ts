#!/usr/bin/env -S deno run -A

const args = Deno.args;
const isMusl = args.some((arg) => arg.includes("musl"));

const features = isMusl
  ? ["--no-default-features", "--features", "rustls"]
  : [];

// Run cargo test
const testCmd = ["cargo", "test", "--release", ...features, ...args];
const testProcess = new Deno.Command(testCmd[0], {
  args: testCmd.slice(1),
  stdout: "inherit",
  stderr: "inherit",
});

const { code: testCode } = await testProcess.output();
if (testCode !== 0) {
  Deno.exit(testCode);
}

// Run JS tests
const jsTestCmd = [
  "cargo",
  "run",
  "--release",
  ...features,
  ...args,
  "--bin",
  "mdeno",
  "--",
  "test",
  ".",
];
const jsTestProcess = new Deno.Command(jsTestCmd[0], {
  args: jsTestCmd.slice(1),
  stdout: "inherit",
  stderr: "inherit",
});

const { code: jsTestCode } = await jsTestProcess.output();
Deno.exit(jsTestCode);
