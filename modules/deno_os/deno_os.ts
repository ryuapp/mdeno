// Copyright 2018-2025 the Deno authors. MIT license.
// Register OS APIs under __mdeno__.os
// @ts-ignore: mdeno internal API
const __internal = globalThis[Symbol.for("mdeno.internal")];

const noColorValue = __internal.noColor ?? false;

class PermissionStatus {
  #state: string;
  #partial: boolean;

  constructor(state: string = "granted", partial: boolean = false) {
    this.#state = state;
    this.#partial = partial;
  }

  get state(): string {
    return this.#state;
  }

  get partial(): boolean {
    return this.#partial;
  }

  get onchange(): null {
    return null;
  }

  set onchange(_handler: unknown) {
    // Ignore onchange setter
  }
}

// @ts-ignore: mdeno internal API
Object.assign(globalThis.__mdeno__.os, {
  args: __internal.args || [],

  exit: function (code: number): void {
    __internal.exit(code);
  },

  env: {
    get: function (key: string): string | undefined {
      return __internal.env.get(key);
    },
    set: function (key: string, value: string): void {
      __internal.env.set(key, value);
    },
    delete: function (key: string): void {
      __internal.env.delete(key);
    },
    has: function (key: string): boolean {
      return __internal.env.has(key);
    },
    toObject: function (): Record<string, string> {
      return __internal.env.toObject();
    },
  },

  get noColor(): boolean {
    return noColorValue;
  },

  get build(): unknown {
    return __internal.build;
  },

  PermissionStatus: PermissionStatus,
});
