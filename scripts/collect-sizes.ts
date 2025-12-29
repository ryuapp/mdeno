/**
 * Collect binary sizes for CI
 * Outputs JSON with sizes of mdeno, mdenort, and compiled example
 *
 * Usage:
 *   deno run -R scripts/collect-sizes.ts [target]
 */

/**
 * Format bytes to human-readable size
 */
function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

function getFileSize(path: string): number | null {
  try {
    const stat = Deno.statSync(path);
    return stat.size;
  } catch {
    return null;
  }
}

function getBinaryExt(target?: string): ".exe" | "" {
  if (target && target.includes("windows")) {
    return ".exe";
  }
  return Deno.build.os === "windows" ? ".exe" : "";
}

function main() {
  const target = Deno.args[0];
  const ext = getBinaryExt(target);
  const targetPath = target ? `${target}/` : "";

  const mdenoPath = `target/${targetPath}release/mdeno${ext}`;
  const mdenortPath = `target/${targetPath}release/mdenort${ext}`;
  const examplePath = `example${ext}`;

  const mdenoSize = getFileSize(mdenoPath);
  const mdenortSize = getFileSize(mdenortPath);
  const exampleSize = getFileSize(examplePath);

  const output = {
    target: target || "unknown",
    mdeno: mdenoSize
      ? {
        size: formatBytes(mdenoSize),
        size_bytes: mdenoSize,
      }
      : null,
    mdenort: mdenortSize
      ? {
        size: formatBytes(mdenortSize),
        size_bytes: mdenortSize,
      }
      : null,
    example: exampleSize
      ? {
        size: formatBytes(exampleSize),
        size_bytes: exampleSize,
      }
      : null,
  };

  console.log(JSON.stringify(output, null, 2));
}

main();
