import { isAbsolute, relative, resolve, sep } from "node:path";

export function resolveArtifactDirectory(value, repoRoot) {
  if (typeof value !== "string" || !isAbsolute(value)) throw new Error("E_CURRENT_WEB_ARTIFACTS_ABSOLUTE_REQUIRED");
  const output = resolve(value);
  const relation = relative(resolve(repoRoot), output);
  if (!relation || !(relation === ".." || relation.startsWith(`..${sep}`) || isAbsolute(relation))) {
    throw new Error("E_CURRENT_WEB_ARTIFACTS_INSIDE_REPO");
  }
  return output;
}
