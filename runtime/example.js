// Navigator API
console.log("Navigator:", navigator.userAgent);

// Deno FileSystem API
const data = Deno.readTextFileSync("README.md");
console.log(data);

// URL API
const url = new URL("https://example.com/path?query=value#hash");
console.log("URL:", url.href);

console.log(Response);
// Fetch API
console.log("\nTesting Fetch API...");
try {
  const promise = fetch("https://ryu.app");
  console.log("Fetch returns:", promise);

  const response = await promise;
  console.log("Response status:", response.status);
  console.log("Response ok:", response.ok);
  const text = await response.text();
  console.log("Response body length:", text.length, "bytes");
} catch (error) {
  console.error("Fetch error:", error.message);
}

console.log("Standalone: ", Deno.build.standalone);
console.log(Deno.args);

Deno.exit(0);
console.log("exit");
