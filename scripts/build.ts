#!/usr/bin/env -S deno run -A

const args = Deno.args;
const isMusl = args.some((arg) => arg.includes("musl"));

const baseCmd = ["cargo", "build", "--release", "--bins"];
const features = isMusl
  ? ["--no-default-features", "--features", "rustls"]
  : [];
const cmd = [...baseCmd, ...features, ...args];

const process = new Deno.Command(cmd[0], {
  args: cmd.slice(1),
  stdout: "inherit",
  stderr: "inherit",
});

const { code } = await process.output();
Deno.exit(code);
