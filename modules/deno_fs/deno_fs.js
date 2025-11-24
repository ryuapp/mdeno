// Copyright 2018-2025 the Deno authors. MIT license.
// Register file system APIs under __mdeno__.fs
const __internal = globalThis[Symbol.for("mdeno.internal")];

const PATHNAME_WIN_RE = /^\/*([A-Za-z]:)(\/|$)/;
const SLASH_WIN_RE = /\//g;
const PERCENT_RE = /%(?![0-9A-Fa-f]{2})/g;

// Convert Windows file URL to path (e.g., file:///C:/path → C:\path)
function pathFromURLWin32(url) {
  let p = url.pathname.replace(PATHNAME_WIN_RE, "$1/");
  p = p.replace(SLASH_WIN_RE, "\\");
  p = p.replace(PERCENT_RE, "%25");
  let path = decodeURIComponent(p);
  if (url.hostname !== "") {
    path = `\\\\${url.hostname}${path}`;
  }
  return path;
}

// Convert POSIX file URL to path (e.g., file:///home/user/path → /home/user/path)
function pathFromURLPosix(url) {
  if (url.hostname !== "") {
    throw new TypeError("Host must be empty");
  }
  return decodeURIComponent(
    url.pathname.replace(PERCENT_RE, "%25"),
  );
}

function pathFromURL(pathOrUrl) {
  if (pathOrUrl instanceof URL) {
    if (pathOrUrl.protocol !== "file:") {
      throw new TypeError("Must be a file URL");
    }

    return navigator.platform === "Win32"
      ? pathFromURLWin32(pathOrUrl)
      : pathFromURLPosix(pathOrUrl);
  }
  return String(pathOrUrl);
}

Object.assign(globalThis.__mdeno__.fs, {
  // https://docs.deno.com/api/deno/~/Deno.readFileSync
  readFileSync(path) {
    path = pathFromURL(path);
    return __internal.fs.readFileSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.readTextFileSync
  readTextFileSync(path) {
    path = pathFromURL(path);
    return __internal.fs.readTextFileSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.writeFileSync
  writeFileSync(path, data, options) {
    path = pathFromURL(path);
    if (typeof data === "string") {
      data = new TextEncoder().encode(data);
    }
    const opts = options ? JSON.stringify(options) : null;
    return __internal.fs.writeFileSync(path, data, opts);
  },

  // https://docs.deno.com/api/deno/~/Deno.writeTextFileSync
  writeTextFileSync(path, text, options) {
    path = pathFromURL(path);
    const opts = options ? JSON.stringify(options) : null;
    return __internal.fs.writeTextFileSync(
      path,
      String(text),
      opts,
    );
  },

  // https://docs.deno.com/api/deno/~/Deno.statSync
  statSync(path) {
    path = pathFromURL(path);
    const result = __internal.fs.statSync(path);
    return JSON.parse(result);
  },

  // https://docs.deno.com/api/deno/~/Deno.mkdirSync
  mkdirSync(path, options) {
    path = pathFromURL(path);
    const opts = options ? JSON.stringify(options) : null;
    return __internal.fs.mkdirSync(path, opts);
  },

  // https://docs.deno.com/api/deno/~/Deno.removeSync
  removeSync(path, options) {
    path = pathFromURL(path);
    const opts = options ? JSON.stringify(options) : null;
    return __internal.fs.removeSync(path, opts);
  },

  // https://docs.deno.com/api/deno/~/Deno.copyFileSync
  copyFileSync(fromPath, toPath) {
    fromPath = pathFromURL(fromPath);
    toPath = pathFromURL(toPath);
    return __internal.fs.copyFileSync(fromPath, toPath);
  },

  // https://docs.deno.com/api/deno/~/Deno.lstatSync
  lstatSync(path) {
    path = pathFromURL(path);
    const result = __internal.fs.lstatSync(path);
    return JSON.parse(result);
  },

  // https://docs.deno.com/api/deno/~/Deno.readDirSync
  readDirSync(path) {
    path = pathFromURL(path);
    const result = __internal.fs.readDirSync(path);
    const entries = JSON.parse(result);
    return entries;
  },

  // https://docs.deno.com/api/deno/~/Deno.renameSync
  renameSync(oldpath, newpath) {
    oldpath = pathFromURL(oldpath);
    newpath = pathFromURL(newpath);
    return __internal.fs.renameSync(oldpath, newpath);
  },

  // https://docs.deno.com/api/deno/~/Deno.realPathSync
  realPathSync(path) {
    path = pathFromURL(path);
    return __internal.fs.realPathSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.truncateSync
  truncateSync(path, len) {
    path = pathFromURL(path);
    return __internal.fs.truncateSync(path, len);
  },

  // https://docs.deno.com/api/deno/~/Deno.makeTempDirSync
  makeTempDirSync(options) {
    const opts = options ? JSON.stringify(options) : null;
    return __internal.fs.makeTempDirSync(opts);
  },

  // https://docs.deno.com/api/deno/~/Deno.makeTempFileSync
  makeTempFileSync(options) {
    const opts = options ? JSON.stringify(options) : null;
    return __internal.fs.makeTempFileSync(opts);
  },
});
