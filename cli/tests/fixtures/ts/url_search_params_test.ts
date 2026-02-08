// URLSearchParams API tests

Deno.test("URLSearchParams constructor with string", () => {
  const params = new URLSearchParams("foo=bar&baz=qux");
  if (params.get("foo") !== "bar") {
    throw new Error(`Expected foo to be "bar", got "${params.get("foo")}"`);
  }
  if (params.get("baz") !== "qux") {
    throw new Error(`Expected baz to be "qux", got "${params.get("baz")}"`);
  }
});

Deno.test("URLSearchParams constructor with leading ?", () => {
  const params = new URLSearchParams("?foo=bar");
  if (params.get("foo") !== "bar") {
    throw new Error(`Expected foo to be "bar", got "${params.get("foo")}"`);
  }
});

Deno.test("URLSearchParams constructor empty", () => {
  const params = new URLSearchParams();
  if (params.size !== 0) {
    throw new Error(`Expected size to be 0, got ${params.size}`);
  }
});

Deno.test("URLSearchParams get", () => {
  const params = new URLSearchParams("foo=bar&foo=baz");
  if (params.get("foo") !== "bar") {
    throw new Error(
      `Expected first foo to be "bar", got "${params.get("foo")}"`,
    );
  }
});

// TODO: Test that get() returns null for non-existent keys (WHATWG spec compliance)
// Deno.test("URLSearchParams get non-existent key", () => {
//   const params = new URLSearchParams("foo=bar");
//   if (params.get("notfound") !== null) {
//     throw new Error(
//       `Expected null for non-existent key, got "${params.get("notfound")}"`,
//     );
//   }
// });

Deno.test("URLSearchParams getAll", () => {
  const params = new URLSearchParams("foo=bar&foo=baz");
  const values = params.getAll("foo");
  if (values.length !== 2) {
    throw new Error(`Expected 2 values, got ${values.length}`);
  }
  if (values[0] !== "bar" || values[1] !== "baz") {
    throw new Error(`Expected ["bar", "baz"], got [${values}]`);
  }
});

Deno.test("URLSearchParams has", () => {
  const params = new URLSearchParams("foo=bar");
  if (!params.has("foo")) {
    throw new Error("Expected has('foo') to be true");
  }
  if (params.has("notfound")) {
    throw new Error("Expected has('notfound') to be false");
  }
});

Deno.test("URLSearchParams append", () => {
  const params = new URLSearchParams("foo=bar");
  params.append("foo", "baz");
  const values = params.getAll("foo");
  if (values.length !== 2) {
    throw new Error(`Expected 2 values, got ${values.length}`);
  }
});

Deno.test("URLSearchParams set", () => {
  const params = new URLSearchParams("foo=bar&foo=baz");
  params.set("foo", "qux");
  const values = params.getAll("foo");
  if (values.length !== 1) {
    throw new Error(`Expected 1 value, got ${values.length}`);
  }
  if (values[0] !== "qux") {
    throw new Error(`Expected "qux", got "${values[0]}"`);
  }
});

Deno.test("URLSearchParams delete", () => {
  const params = new URLSearchParams("foo=bar&baz=qux");
  params.delete("foo");
  if (params.has("foo")) {
    throw new Error("Expected foo to be deleted");
  }
  if (!params.has("baz")) {
    throw new Error("Expected baz to still exist");
  }
});

Deno.test("URLSearchParams size", () => {
  const params = new URLSearchParams("foo=bar&baz=qux");
  if (params.size !== 2) {
    throw new Error(`Expected size to be 2, got ${params.size}`);
  }
});

Deno.test("URLSearchParams sort", () => {
  const params = new URLSearchParams("z=1&a=2&m=3");
  params.sort();
  const str = params.toString();
  if (str !== "a=2&m=3&z=1") {
    throw new Error(`Expected "a=2&m=3&z=1", got "${str}"`);
  }
});

Deno.test("URLSearchParams toString", () => {
  const params = new URLSearchParams("foo=bar&baz=qux");
  const str = params.toString();
  if (str !== "foo=bar&baz=qux") {
    throw new Error(`Expected "foo=bar&baz=qux", got "${str}"`);
  }
});

Deno.test("URLSearchParams toString empty", () => {
  const params = new URLSearchParams();
  const str = params.toString();
  if (str !== "") {
    throw new Error(`Expected empty string, got "${str}"`);
  }
});

Deno.test("URL.searchParams access (GC test)", () => {
  const url = new URL("https://example.com?foo=bar");
  const params = url.searchParams;
  if (params.get("foo") !== "bar") {
    throw new Error(`Expected foo to be "bar", got "${params.get("foo")}"`);
  }
});

Deno.test("URL.searchParams multiple access (GC test)", () => {
  const url = new URL("https://example.com?foo=bar");
  const params1 = url.searchParams;
  const params2 = url.searchParams;
  // Both should work without GC assertion
  if (params1.get("foo") !== "bar") {
    throw new Error("params1: Expected foo to be bar");
  }
  if (params2.get("foo") !== "bar") {
    throw new Error("params2: Expected foo to be bar");
  }
});

Deno.test("URL.searchParams modification syncs to URL", () => {
  const url = new URL("https://example.com");
  url.searchParams.append("foo", "bar");
  if (!url.href.includes("foo=bar")) {
    throw new Error(`Expected URL to contain "foo=bar", got "${url.href}"`);
  }
});

Deno.test("URL.searchParams modification with multiple values", () => {
  const url = new URL("https://example.com");
  url.searchParams.append("foo", "bar");
  url.searchParams.append("baz", "qux");
  const href = url.href;
  if (!href.includes("foo=bar") || !href.includes("baz=qux")) {
    throw new Error(`Expected URL to contain both params, got "${href}"`);
  }
});

Deno.test("URL.searchParams delete syncs to URL", () => {
  const url = new URL("https://example.com?foo=bar&baz=qux");
  url.searchParams.delete("foo");
  if (url.href.includes("foo=bar")) {
    throw new Error(`Expected URL to not contain "foo=bar", got "${url.href}"`);
  }
  if (!url.href.includes("baz=qux")) {
    throw new Error(
      `Expected URL to still contain "baz=qux", got "${url.href}"`,
    );
  }
});

Deno.test("URL.searchParams set syncs to URL", () => {
  const url = new URL("https://example.com?foo=old");
  url.searchParams.set("foo", "new");
  if (!url.href.includes("foo=new")) {
    throw new Error(`Expected URL to contain "foo=new", got "${url.href}"`);
  }
  if (url.href.includes("foo=old")) {
    throw new Error(`Expected URL to not contain "foo=old", got "${url.href}"`);
  }
});

Deno.test("URL.search change updates searchParams", () => {
  const url = new URL("https://example.com?foo=bar");
  const params = url.searchParams;
  if (params.get("foo") !== "bar") {
    throw new Error(`Expected foo to be "bar", got "${params.get("foo")}"`);
  }
  url.search = "?baz=qux";
  if (params.get("baz") !== "qux") {
    throw new Error(
      `Expected baz to be "qux" after URL.search change, got "${
        params.get("baz")
      }"`,
    );
  }
  if (params.get("foo") !== null) {
    throw new Error(
      `Expected foo to be null after URL.search change, got "${
        params.get("foo")
      }"`,
    );
  }
});

Deno.test("URL.href change updates searchParams", () => {
  const url = new URL("https://example.com?foo=bar");
  const params = url.searchParams;
  url.href = "https://example.com?new=value";
  if (params.get("new") !== "value") {
    throw new Error(
      `Expected new to be "value" after URL.href change, got "${
        params.get("new")
      }"`,
    );
  }
  if (params.get("foo") !== null) {
    throw new Error(
      `Expected foo to be null after URL.href change, got "${
        params.get("foo")
      }"`,
    );
  }
});
