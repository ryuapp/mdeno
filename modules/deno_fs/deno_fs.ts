// Copyright 2018-2025 the Deno authors. MIT license.
// Register file system APIs under __mdeno__.fs
// @ts-ignore: mdeno internal API
const __internal = globalThis[Symbol.for("mdeno.internal")];

const PATHNAME_WIN_RE = /^\/*([A-Za-z]:)(\/|$)/;
const SLASH_WIN_RE = /\//g;
const PERCENT_RE = /%(?![0-9A-Fa-f]{2})/g;

// Convert Windows file URL to path (e.g., file:///C:/path → C:\path)
function pathFromURLWin32(url: URL): string {
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
function pathFromURLPosix(url: URL): string {
  if (url.hostname !== "") {
    throw new TypeError("Host must be empty");
  }
  return decodeURIComponent(
    url.pathname.replace(PERCENT_RE, "%25"),
  );
}

function pathFromURL(pathOrUrl: string | URL): string {
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

// @ts-ignore: mdeno internal API
Object.assign(globalThis.__mdeno__.fs, {
  // https://docs.deno.com/api/deno/~/Deno.cwd
  cwd(): string {
    return __internal.fs.cwd();
  },

  // https://docs.deno.com/api/deno/~/Deno.readFileSync
  readFileSync(path: string | URL): Uint8Array {
    path = pathFromURL(path);
    return __internal.fs.readFileSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.readTextFileSync
  readTextFileSync(path: string | URL): string {
    path = pathFromURL(path);
    return __internal.fs.readTextFileSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.writeFileSync
  writeFileSync(
    path: string | URL,
    data: Uint8Array | string,
    options?: unknown,
  ): void {
    path = pathFromURL(path);
    if (typeof data === "string") {
      data = new TextEncoder().encode(data);
    }
    return __internal.fs.writeFileSync(path, data, options);
  },

  // https://docs.deno.com/api/deno/~/Deno.writeTextFileSync
  writeTextFileSync(path: string | URL, text: string, options?: unknown): void {
    path = pathFromURL(path);
    return __internal.fs.writeTextFileSync(path, String(text), options);
  },

  // https://docs.deno.com/api/deno/~/Deno.statSync
  statSync(path: string | URL): unknown {
    path = pathFromURL(path);
    return __internal.fs.statSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.mkdirSync
  mkdirSync(path: string | URL, options?: unknown): void {
    path = pathFromURL(path);
    return __internal.fs.mkdirSync(path, options);
  },

  // https://docs.deno.com/api/deno/~/Deno.removeSync
  removeSync(path: string | URL, options?: unknown): void {
    path = pathFromURL(path);
    return __internal.fs.removeSync(path, options);
  },

  // https://docs.deno.com/api/deno/~/Deno.copyFileSync
  copyFileSync(fromPath: string | URL, toPath: string | URL): void {
    fromPath = pathFromURL(fromPath);
    toPath = pathFromURL(toPath);
    return __internal.fs.copyFileSync(fromPath, toPath);
  },

  // https://docs.deno.com/api/deno/~/Deno.lstatSync
  lstatSync(path: string | URL): unknown {
    path = pathFromURL(path);
    return __internal.fs.lstatSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.readDirSync
  readDirSync(path: string | URL): unknown {
    path = pathFromURL(path);
    return __internal.fs.readDirSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.renameSync
  renameSync(oldpath: string | URL, newpath: string | URL): void {
    oldpath = pathFromURL(oldpath);
    newpath = pathFromURL(newpath);
    return __internal.fs.renameSync(oldpath, newpath);
  },

  // https://docs.deno.com/api/deno/~/Deno.realPathSync
  realPathSync(path: string | URL): string {
    path = pathFromURL(path);
    return __internal.fs.realPathSync(path);
  },

  // https://docs.deno.com/api/deno/~/Deno.truncateSync
  truncateSync(path: string | URL, len?: number): void {
    path = pathFromURL(path);
    return __internal.fs.truncateSync(path, len);
  },

  // https://docs.deno.com/api/deno/~/Deno.makeTempDirSync
  makeTempDirSync(options?: unknown): string {
    return __internal.fs.makeTempDirSync(options);
  },

  // https://docs.deno.com/api/deno/~/Deno.makeTempFileSync
  makeTempFileSync(options?: unknown): string {
    return __internal.fs.makeTempFileSync(options);
  },
});
