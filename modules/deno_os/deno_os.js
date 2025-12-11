// Copyright 2018-2025 the Deno authors. MIT license.
// Register OS APIs under __mdeno__.os
const __internal = globalThis[Symbol.for("mdeno.internal")];

const noColorValue = __internal.noColor ?? false;

class PermissionStatus {
  #state;
  #partial;

  constructor(state = "granted", partial = false) {
    this.#state = state;
    this.#partial = partial;
  }

  get state() {
    return this.#state;
  }

  get partial() {
    return this.#partial;
  }

  get onchange() {
    return null;
  }

  set onchange(_handler) {
    // Ignore onchange setter
  }
}

Object.assign(globalThis.__mdeno__.os, {
  args: __internal.args || [],

  exit: function (code) {
    __internal.exit(code);
  },

  env: {
    get: function (key) {
      return __internal.env.get(key);
    },
    set: function (key, value) {
      __internal.env.set(key, value);
    },
    delete: function (key) {
      __internal.env.delete(key);
    },
    has: function (key) {
      return __internal.env.has(key);
    },
    toObject: function () {
      return __internal.env.toObject();
    },
  },

  get noColor() {
    return noColorValue;
  },

  get build() {
    return __internal.build;
  },

  PermissionStatus: PermissionStatus,
});
