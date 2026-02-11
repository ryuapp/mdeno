// @ts-ignore: mdeno internal API
const __internal = globalThis[Symbol.for("mdeno.internal")];

class Navigator {
  userAgent: string;
  platform: string;
  language: string;
  languages: string[];

  constructor() {
    this.userAgent = "mdeno";
    this.platform = __internal.platform;
    this.language = __internal.language;
    this.languages = [__internal.language];
  }
}

// @ts-ignore: partial Navigator implementation
globalThis.navigator = new Navigator();
