// @ts-ignore: mdeno internal API
const __internal = globalThis[Symbol.for("mdeno.internal")];

function formatValue(arg: unknown): string {
  if (typeof arg === "string") return arg;
  if (arg === null) return "null";
  if (arg === undefined) return "undefined";
  if (typeof arg === "number" || typeof arg === "boolean") {
    return String(arg);
  }
  if (typeof arg === "function") {
    // Check if it's a class (constructor function)
    const fnStr = arg.toString();
    if (fnStr.startsWith("class ")) {
      const className = arg.name || "anonymous";
      // Check for parent class
      const parent = Object.getPrototypeOf(arg);
      if (parent && parent !== Function.prototype && parent.name) {
        return `[class ${className} extends ${parent.name}]`;
      }
      return `[class ${className}]`;
    }
    return `[Function: ${arg.name || "anonymous"}]`;
  }
  if (typeof arg === "symbol") return arg.toString();

  if (Object.prototype.toString.call(arg) === "[object Date]") {
    return (arg as Date).toISOString();
  }

  // Check for Promise
  if (arg instanceof Promise) {
    return "Promise { <pending> }";
  }

  if (Array.isArray(arg)) {
    if (arg.length >= 2) {
      const items = arg.map((item) => {
        if (typeof item === "string") return `  ${JSON.stringify(item)}`;
        if (item === null) return "  null";
        if (item === undefined) return "  undefined";
        return `  ${String(item)}`;
      }).join(",\n");
      return `[\n${items}\n]`;
    } else {
      const items = arg.map((item) => {
        if (typeof item === "string") return JSON.stringify(item);
        if (item === null) return "null";
        if (item === undefined) return "undefined";
        return String(item);
      }).join(", ");
      return `[ ${items} ]`;
    }
  }

  if (typeof arg === "object") {
    const constructorName = (arg as Record<string, unknown>).constructor?.name;
    if (constructorName && constructorName !== "Object") {
      const ownKeys = Object.getOwnPropertyNames(arg);
      const protoKeys = (arg as Record<string, unknown>).constructor.prototype
        ? Object.getOwnPropertyNames(
          (arg as Record<string, unknown>).constructor.prototype,
        ).filter(
          (key: string) => key !== "constructor",
        )
        : [];
      const allKeys = [...new Set([...ownKeys, ...protoKeys])];

      const props = allKeys
        .filter((key) => !key.startsWith("_"))
        .map((key) => {
          try {
            const val = (arg as Record<string, unknown>)[key];
            if (typeof val === "function") return null;
            const valStr = typeof val === "string" ? `"${val}"` : String(val);
            return `  ${key}: ${valStr}`;
          } catch {
            return null;
          }
        })
        .filter((p) => p !== null)
        .join(",\n");
      return `${constructorName} {\n${props}\n}`;
    }
  }

  try {
    return JSON.stringify(arg, null, 2);
  } catch (_e) {
    return String(arg);
  }
}

globalThis.console = {
  log(...args: unknown[]) {
    const formatted = args.map(formatValue).join(" ");
    __internal.print(formatted);
  },
  error(...args: unknown[]) {
    const formatted = args.map(formatValue).join(" ");
    __internal.print(formatted);
  },
} as Console;
