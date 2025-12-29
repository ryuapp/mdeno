// Navigator API
console.log("Navigator:", navigator.userAgent);

// Deno FileSystem API
const data: string = Deno.readTextFileSync("README.md");
console.log(data);

// URL API
const url: URL = new URL("https://example.com/path?query=value#hash");
console.log("URL:", url.href);

console.log(Response);
// Fetch API
console.log("\nTesting Fetch API...");
try {
  const promise: Promise<Response> = fetch("https://ryu.app");
  console.log("Fetch returns:", promise);

  const response: Response = await promise;
  console.log("Response status:", response.status);
  console.log("Response ok:", response.ok);
  const text: string = await response.text();
  console.log("Response body length:", text.length, "bytes");
} catch (error) {
  console.error("Fetch error:", (error as Error).message);
}

// @ts-ignore Deno API issue
console.log("Standalone:", Deno.build.standalone);
console.log("cwd:", Deno.cwd());
console.log("args:", Deno.args);

Deno.exit(0);
console.log("exit");
