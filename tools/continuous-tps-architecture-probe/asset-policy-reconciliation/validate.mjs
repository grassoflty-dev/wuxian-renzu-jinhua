import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const MANIFEST_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/assets/manifest-v1.json");
const AG13_RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/assets-ag13.json");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json");

const FIVE_STATES = ["verified_original", "verified_redistributable", "pending_review", "local_only", "rejected"];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function repoPath(value) {
  return value.split(path.sep).join("/");
}

function absRepo(relative) {
  return path.join(ROOT, ...relative.split("/"));
}

function evidenceIdsFor(entry) {
  if (entry.id === "wuxian.vendor.shared.threejs.r150") return ["AG13-THREE-MIT-NOTICE", "AG13-THREE-LICENSE-TEXT", "AG13-THREE-MANIFEST-HASH"];
  if (entry.id === "wuxian.vendor.shared.babylonjs.current") return ["AG13-BABYLON-RUNTIME-REFERENCE", "AG13-BABYLON-MANIFEST-HASH"];
  if (entry.id === "wuxian.image.shared.cenyao_portrait.wan_v1") return ["AG13-PORTRAIT-DECLARATION", "AG13-PORTRAIT-MANIFEST-HASH"];
  if (entry.kind === "font") return ["AG13-FONT-CSS-STACK", "AG13-FONT-NO-PACKAGED-FILE"];
  if (entry.bundle === "legacy-hive2d") return ["AG13-HISTORICAL-HIVE2D-DEVONLY", `AG13-MANIFEST-${entry.id}`];
  if (entry.bundle === "unverified-experiments") return ["AG13-RAW-HIVE2D-DEVONLY", `AG13-MANIFEST-${entry.id}`];
  if (entry.kind === "logical") return ["AG13-LOGICAL-REQUIREMENT", `AG13-MANIFEST-${entry.id}`];
  return [`AG13-MANIFEST-${entry.id}`];
}

function sourceEvidenceFor(entry) {
  if (entry.id === "wuxian.vendor.shared.threejs.r150") {
    return {
      authorOrSource: "Three.js Authors; local vendor copy is identified as Three.js r150",
      sourceEvidencePointers: ["server-rs/release-ui/THIRD_PARTY_NOTICES.md:3-9"],
      licenseEvidencePointers: ["server-rs/release-ui/vendor/THREE-LICENSE.txt:1-16"],
      sourceRedistributionProof: {
        status: "present_in_existing_ag13_evidence_but_not_new_for_ag41",
        pointer: "server-rs/release-ui/vendor/THREE-LICENSE.txt:7-9",
        scope: "MIT text explicitly permits use, copy, modify, publish, distribute, sublicense, and sell"
      },
      modificationRights: true,
      explicitPublicRedistributionPermission: true
    };
  }
  if (entry.id === "wuxian.vendor.shared.babylonjs.current") {
    return {
      authorOrSource: "Unknown from repository evidence",
      sourceEvidencePointers: ["server-rs/release-ui/index.html:34", "server-rs/release-ui/js/renderer-boot.js:8"],
      licenseEvidencePointers: [],
      sourceRedistributionProof: { status: "absent", pointer: null, scope: "No matching repository license/provenance record" },
      modificationRights: false,
      explicitPublicRedistributionPermission: false
    };
  }
  if (entry.id === "wuxian.image.shared.cenyao_portrait.wan_v1") {
    return {
      authorOrSource: "Release notice claims procedural/original art, but no item-level author record is present",
      sourceEvidencePointers: ["server-rs/release-ui/THIRD_PARTY_NOTICES.md", "server-rs/release-ui/index.html:29"],
      licenseEvidencePointers: [],
      sourceRedistributionProof: { status: "absent", pointer: null, scope: "Declaration-only evidence is insufficient" },
      modificationRights: false,
      explicitPublicRedistributionPermission: false
    };
  }
  if (entry.kind === "font") {
    return {
      authorOrSource: "Platform/system font stack declared by CSS; no font file is packaged",
      sourceEvidencePointers: ["server-rs/release-ui/css/style.css:19-20"],
      licenseEvidencePointers: [],
      sourceRedistributionProof: { status: "not_applicable_to_packaged_assets", pointer: null, scope: "System fonts are not bundled assets" },
      modificationRights: false,
      explicitPublicRedistributionPermission: false
    };
  }
  if (entry.bundle === "legacy-hive2d") {
    return {
      authorOrSource: "Unknown; historical side2d asset retained for audit only",
      sourceEvidencePointers: ["docs/CONTINUOUS_ASSET_PROVENANCE_REVIEW.md:26", "server-rs/release-ui/js/hive-side2d.js:64-68"],
      licenseEvidencePointers: [],
      sourceRedistributionProof: { status: "absent", pointer: null, scope: "No item-level authorship or redistribution record" },
      modificationRights: false,
      explicitPublicRedistributionPermission: false
    };
  }
  if (entry.bundle === "unverified-experiments") {
    return {
      authorOrSource: "Unknown; raw/experimental item with no authorship evidence",
      sourceEvidencePointers: ["docs/CONTINUOUS_ASSET_PROVENANCE_REVIEW.md:27"],
      licenseEvidencePointers: [],
      sourceRedistributionProof: { status: "absent", pointer: null, scope: "Names and file presence are not redistribution proof" },
      modificationRights: false,
      explicitPublicRedistributionPermission: false
    };
  }
  return {
    authorOrSource: "Future/user-provided Grey Hive requirement; authorization not supplied",
    sourceEvidencePointers: ["docs/CONTINUOUS_ASSET_PROVENANCE_REVIEW.md:31-33", `tools/continuous-tps-architecture-probe/assets/manifest-v1.json#${entry.id}`],
    licenseEvidencePointers: [],
    sourceRedistributionProof: { status: "absent", pointer: null, scope: "Requirement-only entry has no source file or authorization" },
    modificationRights: false,
    explicitPublicRedistributionPermission: false
  };
}

function actualUiReferences(entry) {
  const references = Array.isArray(entry.metadata?.referencedBy) ? [...entry.metadata.referencedBy] : [];
  if (entry.id.startsWith("wuxian.font.")) {
    references.push("server-rs/release-ui/css/style.css:19-20");
  }
  return [...new Set(references)].sort();
}

function categoryFor(entry) {
  if (entry.id === "wuxian.vendor.shared.babylonjs.current") return "babylon";
  if (entry.id === "wuxian.vendor.shared.threejs.r150") return "three";
  if (entry.kind === "font") return "fonts";
  if (entry.id === "wuxian.image.shared.cenyao_portrait.wan_v1") return "grey_hive_portrait";
  if (entry.bundle === "legacy-hive2d") return "historical_hive2d";
  if (entry.bundle === "unverified-experiments") return "raw_hive2d";
  return "logical_requirements";
}

function stateFor(entry) {
  if (entry.kind === "font") return "local_only";
  // AG-13's publish_candidate is not promoted: AG-41 has no new item-level evidence.
  return "pending_review";
}

function reasonFor(entry, state) {
  if (state === "local_only") return "System-stack fallback only; no packaged font file is a public asset and no system font is bundled.";
  if (entry.id === "wuxian.vendor.shared.threejs.r150") return "Legacy publish_candidate retained as pending_review because AG-41 adds no new item-level evidence; existing MIT evidence is recorded but not upgraded.";
  if (entry.id === "wuxian.vendor.shared.babylonjs.current") return "Required active runtime has no repository license/provenance proof and no fallback.";
  if (entry.id === "wuxian.image.shared.cenyao_portrait.wan_v1") return "Declaration-only procedural/original claim lacks item-level authorship, modification, and source redistribution proof.";
  if (entry.bundle === "legacy-hive2d") return "Historical devOnly item with no item-level rights chain; retained for audit and excluded from public manifest.";
  if (entry.bundle === "unverified-experiments") return "Raw/experimental devOnly item with no item-level rights chain; retained for audit and excluded from public manifest.";
  return "Requirement-only item has no source file, author record, license, or authorization package.";
}

function mapEntry(entry) {
  const state = stateFor(entry);
  const evidence = sourceEvidenceFor(entry);
  const uiReferences = actualUiReferences(entry);
  const physical = entry.path !== null;
  let fileCheck = { exists: null, sizeMatchesAg13: null, sha256MatchesAg13: null, actualSizeBytes: null, actualSha256: null };
  if (physical) {
    const file = absRepo(entry.path);
    const exists = fs.existsSync(file);
    fileCheck = {
      exists,
      sizeMatchesAg13: exists && fs.statSync(file).size === entry.sizeBytes,
      sha256MatchesAg13: exists && sha256(file) === entry.sha256,
      actualSizeBytes: exists ? fs.statSync(file).size : null,
      actualSha256: exists ? sha256(file) : null
    };
  }
  return {
    assetId: entry.id,
    category: categoryFor(entry),
    kind: entry.kind,
    path: entry.path,
    sha256: entry.sha256,
    sizeBytes: entry.sizeBytes,
    required: entry.required,
    preload: entry.preload,
    fallbackAssetId: entry.fallbackAssetId,
    fallbackResolution: entry.required ? (entry.fallbackAssetId ? "declared" : "unresolved_required_no_fallback") : "not_required",
    devOnly: entry.devOnly,
    legacyAg13Status: entry.reviewStatus,
    actualUiReferences: uiReferences,
    actualUiReferenceStatus: uiReferences.length ? "referenced_or_declared_in_ag13_scope" : "no_reference_in_ag13_scope",
    evidenceIds: evidenceIdsFor(entry),
    authorOrSource: evidence.authorOrSource,
    sourceEvidencePointers: evidence.sourceEvidencePointers,
    licenseEvidencePointers: evidence.licenseEvidencePointers,
    sourceRedistributionProof: evidence.sourceRedistributionProof,
    modificationRights: evidence.modificationRights,
    explicitPublicRedistributionPermission: evidence.explicitPublicRedistributionPermission,
    policyState: state,
    policyReason: reasonFor(entry, state),
    publicManifestEligible: ["verified_original", "verified_redistributable"].includes(state),
    fileCheck
  };
}

function countBy(items, selector) {
  const counts = {};
  for (const item of items) counts[selector(item)] = (counts[selector(item)] ?? 0) + 1;
  return counts;
}

function validate() {
  const manifest = readJson(MANIFEST_PATH);
  const ag13 = readJson(AG13_RESULT_PATH);
  const entries = manifest.entries;
  const mapped = entries.map(mapEntry);
  const physical = mapped.filter((item) => item.path !== null);
  const requiredUnresolved = mapped.filter((item) => item.required && item.fallbackResolution === "unresolved_required_no_fallback");
  const verified = mapped.filter((item) => ["verified_original", "verified_redistributable"].includes(item.policyState));
  const ag13EntryContract = entries.length === 35
    && ag13.inventory?.manifestEntries === 35
    && ag13.inventory?.fileEntries === 20
    && Array.isArray(ag13.inventory?.unmanifestedFiles)
    && ag13.inventory.unmanifestedFiles.length === 0
    && entries.every((entry) => typeof entry.id === "string" && (entry.path === null || typeof entry.sha256 === "string") && (entry.path === null || Number.isInteger(entry.sizeBytes)));
  const hashesAndPathsPass = mapped.every((item) => item.path === null || (item.fileCheck.exists && item.fileCheck.sizeMatchesAg13 && item.fileCheck.sha256MatchesAg13));
  const evidencePass = verified.every((item) => item.evidenceIds.length > 0 && item.sourceRedistributionProof.status === "present" && item.sourceRedistributionProof.pointer);
  const publicManifest = verified.map((item) => item.assetId);
  const statusCounts = countBy(mapped, (item) => item.policyState);
  const categorySummaries = {};
  for (const category of ["babylon", "three", "fonts", "grey_hive_portrait", "historical_hive2d", "raw_hive2d", "logical_requirements"]) {
    const items = mapped.filter((item) => item.category === category);
    categorySummaries[category] = { entries: items.length, physicalFiles: items.filter((item) => item.path !== null).length, statusCounts: countBy(items, (item) => item.policyState), assetIds: items.map((item) => item.assetId) };
  }
  const officialPublishEligible = requiredUnresolved.length === 0
    && physical.length > 0
    && physical.every((item) => ["verified_original", "verified_redistributable"].includes(item.policyState))
    && mapped.every((item) => !item.required || item.publicManifestEligible);
  const checks = {
    ag13Scope35Entries20Files: ag13EntryContract,
    idsPathsHashesMatchAg13: hashesAndPathsPass,
    allFiveStatesValid: mapped.every((item) => FIVE_STATES.includes(item.policyState)),
    everyEntryHasEvidenceIds: mapped.every((item) => item.evidenceIds.length > 0),
    verifiedEvidenceAndRedistributionProof: evidencePass,
    pendingLocalRejectedExcludedFromPublicManifest: mapped.filter((item) => !item.publicManifestEligible).every((item) => !publicManifest.includes(item.assetId)),
    noUnresolvedRequiredFallback: requiredUnresolved.length === 0,
    requiredFallbackBlockRecorded: requiredUnresolved.length > 0 && requiredUnresolved.every((item) => item.policyState === "pending_review" && item.policyReason.length > 0),
    officialPublishEligible
  };
  const pass = Object.entries(checks)
    .filter(([key]) => !["officialPublishEligible", "noUnresolvedRequiredFallback"].includes(key))
    .every(([, value]) => value);
  const result = {
    schema: "continuous_asset_policy_reconciliation_ag41_v1",
    status: pass ? "policy_reconciliation_passed_with_publish_block" : "failed",
    pass,
    machinePass: pass,
    scope: "read-only reconciliation of exactly AG-13's 35 manifest entries and 20 physical files",
    realAssetsModified: false,
    ag13Baseline: {
      resultPath: "tools/continuous-tps-architecture-probe/results/assets-ag13.json",
      status: ag13.status,
      pass: ag13.pass,
      manifestEntries: ag13.inventory?.manifestEntries,
      fileEntries: ag13.inventory?.fileEntries,
      unmanifestedFiles: ag13.inventory?.unmanifestedFiles ?? []
    },
    policy: {
      states: FIVE_STATES,
      mapping: {
        review_required: "pending_review unless an explicit local-only system-stack policy applies",
        publish_candidate: "pending_review without new AG-41 item-level evidence",
        devOnly: "retained as an AG-13 fact; historical/raw files remain pending_review and are not public",
        system_font_stack: "local_only",
        verified_original_or_redistributable: "not assigned without item-level rights, modification, and original-source redistribution proof"
      },
      statusCounts,
      publicManifestAssetIds: publicManifest
    },
    coverage: {
      manifestEntries: mapped.length,
      physicalFiles: physical.length,
      logicalEntries: mapped.length - physical.length,
      categories: categorySummaries
    },
    entries: mapped,
    requiredFallback: {
      requiredEntries: mapped.filter((item) => item.required).map((item) => item.assetId),
      unresolved: requiredUnresolved,
      allRequiredFallbacksResolved: requiredUnresolved.length === 0
    },
    blockingAssets: mapped.filter((item) => item.required || !item.publicManifestEligible).map((item) => ({ assetId: item.assetId, policyState: item.policyState, required: item.required, reason: item.policyReason })),
    officialPublishEligible,
    checks,
    nextTp08RegistryAuthorizationPackageWhitelist: [
      "tools/continuous-tps-architecture-probe/asset-policy-reconciliation/tp08-asset-registry-v1.json",
      "tools/continuous-tps-architecture-probe/asset-policy-reconciliation/tp08-item-authorization-evidence-v1.json",
      "tools/continuous-tps-architecture-probe/asset-policy-reconciliation/tp08-public-manifest-v1.json",
      "docs/CONTINUOUS_TP08_ASSET_REGISTRY_AUTHORIZATION_REVIEW.md"
    ],
    formalDecision: "Gate G and overall Architecture Gate remain blocked/no_go; AG-41 is policy reconciliation only."
  };
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ status: result.status, pass: result.pass, officialPublishEligible, statusCounts, manifestEntries: mapped.length, physicalFiles: physical.length }));
  if (!result.pass) process.exitCode = 1;
}

validate();
