import fs from "node:fs";
import path from "node:path";

export const repoRoot = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const relativePath = file => file.slice(repoRoot.length).replace(/^[/\\]+/, "").replaceAll("\\", "/");

function sourceFiles(config) {
  const root = path.join(repoRoot, config.sourceRoots[0]);
  return fs.readdirSync(root).filter(name => name.endsWith(".rs")).sort().map(name => path.join(root, name));
}

function collectSymbols(file, text) {
  const lines = text.split(/\r?\n/);
  const stem = path.basename(file, ".rs");
  const symbols = [];
  let current = null;
  let depth = 0;
  for (let i = 0; i < lines.length; i++) {
    const match = lines[i].match(/\b(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/);
    if (match) {
      if (current) current.endLine = i;
      current = { key: `${stem}::${match[1]}`, path: relativePath(file), symbol: `${stem}::${match[1]}`, startLine: i + 1, endLine: lines.length };
      symbols.push(current);
      depth = 0;
    }
    if (current) {
      depth += (lines[i].match(/{/g) ?? []).length;
      depth -= (lines[i].match(/}/g) ?? []).length;
      if (depth <= 0 && i + 1 > current.startLine) { current.endLine = i + 1; current = null; }
    }
  }
  return symbols;
}

function classifyLine(line) {
  const trimmed = line.trim();
  if (!trimmed || trimmed.startsWith("//") || trimmed.startsWith("/*") || trimmed.startsWith("*") || trimmed.startsWith("*/")) return "comment/string only";
  if (/^([ru#]*["']|[a-zA-Z_][\w]*\s*:\s*[ru#]*["'])/.test(trimmed) && !trimmed.includes("fn ")) return "comment/string only";
  return "executable/static source";
}

function sinkHits(line, lexicon) {
  return lexicon.filter(sink => sink.patterns.some(pattern => new RegExp(pattern, "i").test(line))).map(sink => sink.id);
}

export function scanWorkspace(config) {
  const files = sourceFiles(config);
  const allSymbols = new Map();
  const records = [];
  const fileTexts = new Map();
  for (const file of files) {
    const text = fs.readFileSync(file, "utf8");
    fileTexts.set(file, text);
    for (const symbol of collectSymbols(file, text)) allSymbols.set(symbol.key, symbol);
  }
  const unresolvedEdges = [];
  const edges = [];
  for (const [from, to, kind] of config.configuredEdges) {
    const resolved = allSymbols.has(from) && allSymbols.has(to);
    const dynamic = kind.startsWith("dynamic_");
    const edge = { from, to, kind, resolved, classification: resolved ? "static edge" : dynamic ? "unknown/unresolved dynamic edge" : "unknown/unresolved static edge" };
    edges.push(edge);
    if (!resolved) unresolvedEdges.push(edge);
  }
  const adjacency = new Map();
  for (const edge of edges.filter(edge => edge.resolved)) { if (!adjacency.has(edge.from)) adjacency.set(edge.from, []); adjacency.get(edge.from).push(edge.to); }
  const entryRecords = [];
  const formalEntries = new Set();
  for (const entry of config.formalEntrypoints) {
    const symbol = allSymbols.get(entry.symbol);
    if (!symbol) entryRecords.push({ ...entry, status: "unknown", errorCode: "E_ENTRYPOINT_MISSING" });
    else { formalEntries.add(entry.symbol); entryRecords.push({ ...entry, status: "identified", line: symbol.startLine }); }
  }
  const reachable = new Set();
  const queue = [...formalEntries];
  while (queue.length) { const node = queue.shift(); if (reachable.has(node)) continue; reachable.add(node); for (const next of adjacency.get(node) ?? []) if (!reachable.has(next)) queue.push(next); }
  const pathsTo = target => {
    const paths = [];
    for (const entry of formalEntries) {
      const parent = new Map([[entry, null]]); const q = [entry];
      while (q.length) { const node = q.shift(); if (node === target) break; for (const next of adjacency.get(node) ?? []) if (!parent.has(next)) { parent.set(next, node); q.push(next); } }
      if (parent.has(target)) { const chain = []; let cursor = target; while (cursor !== null) { chain.unshift(cursor); cursor = parent.get(cursor); } paths.push(chain); }
    }
    return paths;
  };
  for (const [file, text] of fileTexts) {
    const lines = text.split(/\r?\n/); const symbols = [...allSymbols.values()].filter(symbol => symbol.path === relativePath(file));
    for (let i = 0; i < lines.length; i++) {
      const sinkIds = sinkHits(lines[i], config.legacySinkLexicon);
      if (!sinkIds.length) continue;
      const symbol = symbols.find(candidate => i + 1 >= candidate.startLine && i + 1 <= candidate.endLine);
      const classification = classifyLine(lines[i]);
      const formalReachable = symbol ? reachable.has(symbol.key) : false;
      const chain = symbol ? pathsTo(symbol.key)[0] ?? [] : [];
      const kind = classification === "comment/string only" ? "comment/string only" : formalReachable ? "reachable formal authority" : symbol ? "unreachable but present" : "unknown/unresolved dynamic edge";
      records.push({ path: relativePath(file), symbol: symbol?.key ?? null, line: i + 1, sinkIds: [...new Set(sinkIds)].sort(), edgeChain: chain, classification: kind, text: lines[i].trim().slice(0, 180) });
    }
  }
  records.sort((a, b) => a.path.localeCompare(b.path) || a.line - b.line || a.classification.localeCompare(b.classification));
  const coverage = config.formalEntrypoints.map(entry => { const identified = entryRecords.find(item => item.id === entry.id)?.status === "identified"; const reachableLegacyHits = records.filter(record => record.classification === "reachable formal authority" && record.edgeChain.includes(entry.symbol)).length; const hasUnresolved = unresolvedEdges.some(edge => edge.from === entry.symbol || edge.from.startsWith(entry.symbol.split("::")[0])); return { id: entry.id, category: entry.category, entry: entry.symbol, identified, reachableLegacyHits, status: reachableLegacyHits > 0 ? "reachable formal authority" : hasUnresolved || identified ? "unknown/unresolved dynamic edge" : "unknown/unresolved dynamic edge" }; });
  const unknownCodes = [];
  if (unresolvedEdges.length) unknownCodes.push("E_DYNAMIC_EDGE_UNRESOLVED");
  if (entryRecords.some(entry => entry.status === "unknown")) unknownCodes.push("E_ENTRYPOINT_MISSING");
  if (config.formalEntrypoints.some(entry => !coverage.find(item => item.id === entry.id)?.identified)) unknownCodes.push("E_GRAPH_INCOMPLETE");
  return { files: files.map(relativePath).sort(), symbols: [...allSymbols.values()].sort((a, b) => a.key.localeCompare(b.key)), edges, entrypoints: entryRecords, reachableNodes: [...reachable].sort(), unresolvedEdges, records, coverage, unknownCodes: [...new Set(unknownCodes)].sort() };
}
