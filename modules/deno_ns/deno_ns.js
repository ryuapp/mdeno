// Copyright 2018-2025 the Deno authors. MIT license.
// Deno namespace binding point - individual modules define their own APIs

const fs = globalThis.__mdeno__.fs;
const os = globalThis.__mdeno__.os;

const permissionStatus = new os.PermissionStatus("granted", false);

const denoNs = {
  // Command line arguments
  args: os.args,

  // Process APIs
  cwd: fs.cwd,

  // File System APIs
  readFileSync: fs.readFileSync,
  readTextFileSync: fs.readTextFileSync,
  writeFileSync: fs.writeFileSync,
  writeTextFileSync: fs.writeTextFileSync,
  statSync: fs.statSync,
  lstatSync: fs.lstatSync,
  mkdirSync: fs.mkdirSync,
  removeSync: fs.removeSync,
  copyFileSync: fs.copyFileSync,
  readDirSync: fs.readDirSync,
  renameSync: fs.renameSync,
  realPathSync: fs.realPathSync,
  truncateSync: fs.truncateSync,
  makeTempDirSync: fs.makeTempDirSync,
  makeTempFileSync: fs.makeTempFileSync,

  // OS APIs
  exit: os.exit,
  env: os.env,

  // Permission APIs - always grant
  permissions: {
    query: (_desc) => Promise.resolve(permissionStatus),
    querySync: (_desc) => permissionStatus,
    revoke: (_desc) => Promise.resolve(permissionStatus),
    revokeSync: (_desc) => permissionStatus,
    request: (_desc) => Promise.resolve(permissionStatus),
    requestSync: (_desc) => permissionStatus,
  },
};

// Add noColor as a getter
Object.defineProperty(denoNs, "noColor", {
  get() {
    return os.noColor;
  },
});

// Add build as a getter
Object.defineProperty(denoNs, "build", {
  get() {
    return os.build;
  },
});

// Add errors namespace
Object.defineProperty(denoNs, "errors", {
  value: globalThis.__mdeno__.errors,
  enumerable: true,
  writable: false,
  configurable: false,
});

// Define globalThis.Deno
Object.defineProperty(globalThis, "Deno", {
  value: denoNs,
  enumerable: false,
  writable: false,
  configurable: false,
});
