const __internal = globalThis[Symbol.for("mdeno.internal")];

// Base64 encoding/decoding
globalThis.btoa = function btoa(data) {
  if (typeof data !== "string") {
    throw new TypeError(
      "Failed to execute 'btoa': The string to be encoded contains characters outside of the Latin1 range.",
    );
  }

  return __internal.encoding.btoa(data);
};

globalThis.atob = function atob(data) {
  if (typeof data !== "string") {
    throw new TypeError(
      "Failed to execute 'atob': 1 argument required, but only 0 present.",
    );
  }

  const result = __internal.encoding.atob(data);

  // Check if result is an error (string starting with ERROR:)
  if (result.startsWith("ERROR: ")) {
    throw new DOMException(result.substring(7), "InvalidCharacterError");
  }

  return result;
};
