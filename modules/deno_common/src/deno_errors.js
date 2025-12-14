// Copyright 2018-2025 the Deno authors. MIT license.

class NotFound extends Error {
  constructor(msg) {
    super(msg);
    this.name = "NotFound";
  }
}

class PermissionDenied extends Error {
  constructor(msg) {
    super(msg);
    this.name = "PermissionDenied";
  }
}

class ConnectionRefused extends Error {
  constructor(msg) {
    super(msg);
    this.name = "ConnectionRefused";
  }
}

class ConnectionReset extends Error {
  constructor(msg) {
    super(msg);
    this.name = "ConnectionReset";
  }
}

class ConnectionAborted extends Error {
  constructor(msg) {
    super(msg);
    this.name = "ConnectionAborted";
  }
}

class NotConnected extends Error {
  constructor(msg) {
    super(msg);
    this.name = "NotConnected";
  }
}

class AddrInUse extends Error {
  constructor(msg) {
    super(msg);
    this.name = "AddrInUse";
  }
}

class AddrNotAvailable extends Error {
  constructor(msg) {
    super(msg);
    this.name = "AddrNotAvailable";
  }
}

class BrokenPipe extends Error {
  constructor(msg) {
    super(msg);
    this.name = "BrokenPipe";
  }
}

class AlreadyExists extends Error {
  constructor(msg) {
    super(msg);
    this.name = "AlreadyExists";
  }
}

class InvalidData extends Error {
  constructor(msg) {
    super(msg);
    this.name = "InvalidData";
  }
}

class TimedOut extends Error {
  constructor(msg) {
    super(msg);
    this.name = "TimedOut";
  }
}

class Interrupted extends Error {
  constructor(msg) {
    super(msg);
    this.name = "Interrupted";
  }
}

class WouldBlock extends Error {
  constructor(msg) {
    super(msg);
    this.name = "WouldBlock";
  }
}

class WriteZero extends Error {
  constructor(msg) {
    super(msg);
    this.name = "WriteZero";
  }
}

class UnexpectedEof extends Error {
  constructor(msg) {
    super(msg);
    this.name = "UnexpectedEof";
  }
}

class BadResource extends Error {
  constructor(msg) {
    super(msg);
    this.name = "BadResource";
  }
}

class Http extends Error {
  constructor(msg) {
    super(msg);
    this.name = "Http";
  }
}

class Busy extends Error {
  constructor(msg) {
    super(msg);
    this.name = "Busy";
  }
}

class NotSupported extends Error {
  constructor(msg) {
    super(msg);
    this.name = "NotSupported";
  }
}

class FilesystemLoop extends Error {
  constructor(msg) {
    super(msg);
    this.name = "FilesystemLoop";
  }
}

class IsADirectory extends Error {
  constructor(msg) {
    super(msg);
    this.name = "IsADirectory";
  }
}

class NetworkUnreachable extends Error {
  constructor(msg) {
    super(msg);
    this.name = "NetworkUnreachable";
  }
}

class NotADirectory extends Error {
  constructor(msg) {
    super(msg);
    this.name = "NotADirectory";
  }
}

// Export all error classes
globalThis.__mdeno__.errors = {
  NotFound,
  PermissionDenied,
  ConnectionRefused,
  ConnectionReset,
  ConnectionAborted,
  NotConnected,
  AddrInUse,
  AddrNotAvailable,
  BrokenPipe,
  AlreadyExists,
  InvalidData,
  TimedOut,
  Interrupted,
  WouldBlock,
  WriteZero,
  UnexpectedEof,
  BadResource,
  Http,
  Busy,
  NotSupported,
  FilesystemLoop,
  IsADirectory,
  NetworkUnreachable,
  NotADirectory,
};
