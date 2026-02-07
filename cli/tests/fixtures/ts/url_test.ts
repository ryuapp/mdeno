// URL API tests

Deno.test("URL constructor with full URL", () => {
  const url = new URL(
    "https://user:pass@example.com:8080/path?query=value#hash",
  );
  if (url.href !== "https://user:pass@example.com:8080/path?query=value#hash") {
    throw new Error(`Expected full URL, got "${url.href}"`);
  }
});

Deno.test("URL constructor with base", () => {
  const url = new URL("/path", "https://example.com");
  if (url.href !== "https://example.com/path") {
    throw new Error(`Expected "https://example.com/path", got "${url.href}"`);
  }
});

Deno.test("URL.protocol getter", () => {
  const url = new URL("https://example.com");
  if (url.protocol !== "https:") {
    throw new Error(`Expected "https:", got "${url.protocol}"`);
  }
});

Deno.test("URL.protocol setter", () => {
  const url = new URL("https://example.com");
  url.protocol = "http";
  if (url.protocol !== "http:") {
    throw new Error(`Expected "http:", got "${url.protocol}"`);
  }
});

Deno.test("URL.hostname getter", () => {
  const url = new URL("https://example.com:8080");
  if (url.hostname !== "example.com") {
    throw new Error(`Expected "example.com", got "${url.hostname}"`);
  }
});

Deno.test("URL.hostname setter", () => {
  const url = new URL("https://example.com");
  url.hostname = "test.com";
  if (url.hostname !== "test.com") {
    throw new Error(`Expected "test.com", got "${url.hostname}"`);
  }
});

Deno.test("URL.port getter", () => {
  const url = new URL("https://example.com:8080");
  if (url.port !== "8080") {
    throw new Error(`Expected "8080", got "${url.port}"`);
  }
});

Deno.test("URL.port setter", () => {
  const url = new URL("https://example.com");
  url.port = "3000";
  if (url.port !== "3000") {
    throw new Error(`Expected "3000", got "${url.port}"`);
  }
});

Deno.test("URL.pathname getter", () => {
  const url = new URL("https://example.com/path/to/resource");
  if (url.pathname !== "/path/to/resource") {
    throw new Error(`Expected "/path/to/resource", got "${url.pathname}"`);
  }
});

Deno.test("URL.pathname setter", () => {
  const url = new URL("https://example.com");
  url.pathname = "/new/path";
  if (url.pathname !== "/new/path") {
    throw new Error(`Expected "/new/path", got "${url.pathname}"`);
  }
});

Deno.test("URL.search getter", () => {
  const url = new URL("https://example.com?foo=bar");
  if (url.search !== "?foo=bar") {
    throw new Error(`Expected "?foo=bar", got "${url.search}"`);
  }
});

Deno.test("URL.search setter", () => {
  const url = new URL("https://example.com");
  url.search = "?key=value";
  if (url.search !== "?key=value") {
    throw new Error(`Expected "?key=value", got "${url.search}"`);
  }
});

Deno.test("URL.hash getter", () => {
  const url = new URL("https://example.com#section");
  if (url.hash !== "#section") {
    throw new Error(`Expected "#section", got "${url.hash}"`);
  }
});

Deno.test("URL.hash setter", () => {
  const url = new URL("https://example.com");
  url.hash = "#anchor";
  if (url.hash !== "#anchor") {
    throw new Error(`Expected "#anchor", got "${url.hash}"`);
  }
});

Deno.test("URL.username getter", () => {
  const url = new URL("https://user@example.com");
  if (url.username !== "user") {
    throw new Error(`Expected "user", got "${url.username}"`);
  }
});

Deno.test("URL.username setter", () => {
  const url = new URL("https://example.com");
  url.username = "admin";
  if (url.username !== "admin") {
    throw new Error(`Expected "admin", got "${url.username}"`);
  }
});

Deno.test("URL.password getter", () => {
  const url = new URL("https://user:pass@example.com");
  if (url.password !== "pass") {
    throw new Error(`Expected "pass", got "${url.password}"`);
  }
});

Deno.test("URL.password setter", () => {
  const url = new URL("https://user@example.com");
  url.password = "secret";
  if (url.password !== "secret") {
    throw new Error(`Expected "secret", got "${url.password}"`);
  }
});

Deno.test("URL.host getter", () => {
  const url = new URL("https://example.com:8080");
  if (url.host !== "example.com:8080") {
    throw new Error(`Expected "example.com:8080", got "${url.host}"`);
  }
});

Deno.test("URL.host setter", () => {
  const url = new URL("https://example.com");
  url.host = "test.com:9000";
  if (url.host !== "test.com:9000") {
    throw new Error(`Expected "test.com:9000", got "${url.host}"`);
  }
});

Deno.test("URL.origin getter", () => {
  const url = new URL("https://example.com:8080/path");
  if (url.origin !== "https://example.com:8080") {
    throw new Error(`Expected "https://example.com:8080", got "${url.origin}"`);
  }
});

Deno.test("URL.href setter", () => {
  const url = new URL("https://example.com");
  url.href = "https://newsite.com/path";
  if (url.href !== "https://newsite.com/path") {
    throw new Error(`Expected "https://newsite.com/path", got "${url.href}"`);
  }
});

Deno.test("URL.toString()", () => {
  const url = new URL("https://example.com/path");
  if (url.toString() !== "https://example.com/path") {
    throw new Error(
      `Expected "https://example.com/path", got "${url.toString()}"`,
    );
  }
});

Deno.test("URL.toJSON()", () => {
  const url = new URL("https://example.com/path");
  if (url.toJSON() !== "https://example.com/path") {
    throw new Error(
      `Expected "https://example.com/path", got "${url.toJSON()}"`,
    );
  }
});

Deno.test("URL.canParse() with valid URL", () => {
  if (!URL.canParse("https://example.com")) {
    throw new Error("Expected URL.canParse to return true for valid URL");
  }
});

Deno.test("URL.canParse() with invalid URL", () => {
  if (URL.canParse("not a url")) {
    throw new Error("Expected URL.canParse to return false for invalid URL");
  }
});

Deno.test("URL.parse() with valid URL", () => {
  const url = URL.parse("https://example.com");
  if (!url) {
    throw new Error("Expected URL.parse to return URL object");
  }
  if (url.href !== "https://example.com/") {
    throw new Error(`Expected "https://example.com/", got "${url.href}"`);
  }
});

Deno.test("URL.parse() with invalid URL", () => {
  const url = URL.parse("not a url");
  if (url !== null) {
    throw new Error("Expected URL.parse to return null for invalid URL");
  }
});
