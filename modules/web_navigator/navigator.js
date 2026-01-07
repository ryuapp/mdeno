const __internal = globalThis[Symbol.for("mdeno.internal")];

class Navigator {
  constructor() {
    this.userAgent = "mdeno";
    this.platform = __internal.platform;
    this.language = __internal.language;
    this.languages = [__internal.language];
  }
}

globalThis.navigator = new Navigator();
