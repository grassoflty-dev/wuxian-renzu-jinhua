/** Explicit presentation-only diagnostics. Production defaults never reserve a sidebar. */
export function developerPresentationEnabled(search: string): boolean {
  const values = new URLSearchParams(search).getAll("developer");
  return values.length === 1 && values[0] === "1";
}
