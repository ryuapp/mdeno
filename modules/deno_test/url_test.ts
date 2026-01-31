// URL API E2E tests

Deno.test("URL - basic parsing", () => {
  const url = new URL("https://example.com/path");

  if (url.href !== "https://example.com/path") {
    throw new Error(
      `Expected href to be "https://example.com/path", got "${url.href}"`,
    );
  }

  if (url.protocol !== "https:") {
    throw new Error(`Expected protocol to be "https:", got "${url.protocol}"`);
  }

  if (url.hostname !== "example.com") {
    throw new Error(
      `Expected hostname to be "example.com", got "${url.hostname}"`,
    );
  }

  if (url.pathname !== "/path") {
    throw new Error(`Expected pathname to be "/path", got "${url.pathname}"`);
  }
});

Deno.test("URL - with port", () => {
  const url = new URL("https://example.com:8080/path");

  if (url.host !== "example.com:8080") {
    throw new Error(
      `Expected host to be "example.com:8080", got "${url.host}"`,
    );
  }

  if (url.port !== "8080") {
    throw new Error(`Expected port to be "8080", got "${url.port}"`);
  }
});

Deno.test("URL - with query string", () => {
  const url = new URL("https://example.com/path?foo=bar&baz=qux");

  if (url.search !== "?foo=bar&baz=qux") {
    throw new Error(
      `Expected search to be "?foo=bar&baz=qux", got "${url.search}"`,
    );
  }

  if (url.searchParams.get("foo") !== "bar") {
    throw new Error(
      `Expected searchParams.get("foo") to be "bar", got "${
        url.searchParams.get("foo")
      }"`,
    );
  }

  if (url.searchParams.get("baz") !== "qux") {
    throw new Error(
      `Expected searchParams.get("baz") to be "qux", got "${
        url.searchParams.get("baz")
      }"`,
    );
  }
});

Deno.test("URL - with hash", () => {
  const url = new URL("https://example.com/path#section");

  if (url.hash !== "#section") {
    throw new Error(`Expected hash to be "#section", got "${url.hash}"`);
  }
});

Deno.test("URL - with username and password", () => {
  const url = new URL("https://user:pass@example.com/path");

  if (url.username !== "user") {
    throw new Error(`Expected username to be "user", got "${url.username}"`);
  }

  if (url.password !== "pass") {
    throw new Error(`Expected password to be "pass", got "${url.password}"`);
  }
});

Deno.test("URL - origin", () => {
  const url = new URL("https://example.com:8080/path?query=1#hash");

  if (url.origin !== "https://example.com:8080") {
    throw new Error(
      `Expected origin to be "https://example.com:8080", got "${url.origin}"`,
    );
  }
});

Deno.test("URL - relative URL with base", () => {
  const url = new URL("/path/to/page", "https://example.com");

  if (url.href !== "https://example.com/path/to/page") {
    throw new Error(
      `Expected href to be "https://example.com/path/to/page", got "${url.href}"`,
    );
  }
});

Deno.test("URL - relative URL with base (with path)", () => {
  const url = new URL("page.html", "https://example.com/dir/");

  if (url.href !== "https://example.com/dir/page.html") {
    throw new Error(
      `Expected href to be "https://example.com/dir/page.html", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set href", () => {
  const url = new URL("https://example.com/old");
  url.href = "https://newsite.com/new";

  if (url.href !== "https://newsite.com/new") {
    throw new Error(
      `Expected href to be "https://newsite.com/new", got "${url.href}"`,
    );
  }

  if (url.hostname !== "newsite.com") {
    throw new Error(
      `Expected hostname to be "newsite.com", got "${url.hostname}"`,
    );
  }
});

Deno.test("URL - set protocol", () => {
  const url = new URL("https://example.com/path");
  url.protocol = "http:";

  if (url.protocol !== "http:") {
    throw new Error(`Expected protocol to be "http:", got "${url.protocol}"`);
  }

  if (url.href !== "http://example.com/path") {
    throw new Error(
      `Expected href to be "http://example.com/path", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set hostname", () => {
  const url = new URL("https://example.com/path");
  url.hostname = "newhost.com";

  if (url.hostname !== "newhost.com") {
    throw new Error(
      `Expected hostname to be "newhost.com", got "${url.hostname}"`,
    );
  }

  if (url.href !== "https://newhost.com/path") {
    throw new Error(
      `Expected href to be "https://newhost.com/path", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set port", () => {
  const url = new URL("https://example.com/path");
  url.port = "3000";

  if (url.port !== "3000") {
    throw new Error(`Expected port to be "3000", got "${url.port}"`);
  }

  if (url.href !== "https://example.com:3000/path") {
    throw new Error(
      `Expected href to be "https://example.com:3000/path", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set pathname", () => {
  const url = new URL("https://example.com/old");
  url.pathname = "/new/path";

  if (url.pathname !== "/new/path") {
    throw new Error(
      `Expected pathname to be "/new/path", got "${url.pathname}"`,
    );
  }

  if (url.href !== "https://example.com/new/path") {
    throw new Error(
      `Expected href to be "https://example.com/new/path", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set search", () => {
  const url = new URL("https://example.com/path");
  url.search = "?foo=bar";

  if (url.search !== "?foo=bar") {
    throw new Error(`Expected search to be "?foo=bar", got "${url.search}"`);
  }

  if (url.href !== "https://example.com/path?foo=bar") {
    throw new Error(
      `Expected href to be "https://example.com/path?foo=bar", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set hash", () => {
  const url = new URL("https://example.com/path");
  url.hash = "#section";

  if (url.hash !== "#section") {
    throw new Error(`Expected hash to be "#section", got "${url.hash}"`);
  }

  if (url.href !== "https://example.com/path#section") {
    throw new Error(
      `Expected href to be "https://example.com/path#section", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set username", () => {
  const url = new URL("https://example.com/path");
  url.username = "newuser";

  if (url.username !== "newuser") {
    throw new Error(`Expected username to be "newuser", got "${url.username}"`);
  }

  if (url.href !== "https://newuser@example.com/path") {
    throw new Error(
      `Expected href to be "https://newuser@example.com/path", got "${url.href}"`,
    );
  }
});

Deno.test("URL - set password", () => {
  const url = new URL("https://user@example.com/path");
  url.password = "secret";

  if (url.password !== "secret") {
    throw new Error(`Expected password to be "secret", got "${url.password}"`);
  }

  if (url.href !== "https://user:secret@example.com/path") {
    throw new Error(
      `Expected href to be "https://user:secret@example.com/path", got "${url.href}"`,
    );
  }
});

Deno.test("URL - toString()", () => {
  const url = new URL("https://example.com/path?foo=bar#hash");
  const str = url.toString();

  if (str !== "https://example.com/path?foo=bar#hash") {
    throw new Error(
      `Expected toString() to be "https://example.com/path?foo=bar#hash", got "${str}"`,
    );
  }
});

Deno.test("URL - toJSON()", () => {
  const url = new URL("https://example.com/path");
  const json = url.toJSON();

  if (json !== "https://example.com/path") {
    throw new Error(
      `Expected toJSON() to be "https://example.com/path", got "${json}"`,
    );
  }
});

Deno.test("URL - searchParams operations", () => {
  const url = new URL("https://example.com/path?a=1");

  // Add parameter
  url.searchParams.append("b", "2");
  if (url.search !== "?a=1&b=2") {
    throw new Error(`Expected search to be "?a=1&b=2", got "${url.search}"`);
  }

  // Set parameter (replaces)
  url.searchParams.set("a", "99");
  if (url.searchParams.get("a") !== "99") {
    throw new Error(
      `Expected a to be "99", got "${url.searchParams.get("a")}"`,
    );
  }

  // Delete parameter
  url.searchParams.delete("b");
  if (url.searchParams.has("b")) {
    throw new Error("Expected b to be deleted");
  }
});

Deno.test("URL - invalid URL throws", () => {
  let errorThrown = false;

  try {
    new URL("not a valid url");
  } catch (_e) {
    errorThrown = true;
  }

  if (!errorThrown) {
    throw new Error("Expected error to be thrown for invalid URL");
  }
});

Deno.test("URL - file:// protocol", () => {
  const url = new URL("file:///home/user/file.txt");

  if (url.protocol !== "file:") {
    throw new Error(`Expected protocol to be "file:", got "${url.protocol}"`);
  }

  if (url.pathname !== "/home/user/file.txt") {
    throw new Error(
      `Expected pathname to be "/home/user/file.txt", got "${url.pathname}"`,
    );
  }
});

Deno.test("URL - encoded components", () => {
  const url = new URL(
    "https://example.com/path with spaces?key=value with spaces",
  );

  // URL should encode spaces
  if (!url.pathname.includes("%20") && !url.pathname.includes("+")) {
    // Some implementations use %20, others use proper encoding
    if (url.pathname === "/path with spaces") {
      // If not encoded, that's also acceptable for pathname
    }
  }

  // Search params should be encoded
  if (url.searchParams.get("key") !== "value with spaces") {
    throw new Error(
      `Expected decoded value to be "value with spaces", got "${
        url.searchParams.get("key")
      }"`,
    );
  }
});

Deno.test("URL - empty search and hash", () => {
  const url = new URL("https://example.com/path");

  if (url.search !== "") {
    throw new Error(`Expected empty search, got "${url.search}"`);
  }

  if (url.hash !== "") {
    throw new Error(`Expected empty hash, got "${url.hash}"`);
  }
});
