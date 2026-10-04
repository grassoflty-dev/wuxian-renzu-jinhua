export const ACCESSIBILITY_PREFERENCES_STORAGE_KEY = "wuxian.accessibility-preferences.v1";

export interface AccessibilityPreferenceState {
  highContrast: boolean;
  reducedMotion: boolean;
}

// Explicit development choices; source Visual Specification §80 requires both controls.
export const TEXT_SCALE_PERCENTAGES = [100, 115, 130] as const;
export const UI_SCALE_PERCENTAGES = [100, 110, 125] as const;
export type DisplayScaleKey = "textScalePercent" | "uiScalePercent";
export interface DisplayScaleState { textScalePercent: number; uiScalePercent: number }
const DEFAULT_SCALE: DisplayScaleState = { textScalePercent: 100, uiScalePercent: 100 };

export type AccessibilityPreferenceKey = keyof AccessibilityPreferenceState;

export interface PreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export interface PreferenceDatasetTarget {
  dataset: {
    contrast?: string;
    motion?: string;
    textScale?: string;
    uiScale?: string;
  };
}

export interface PreferenceToggleControl {
  setPressed(pressed: boolean): void;
  onToggle(handler: () => void): void;
}

const DEFAULT_STATE: AccessibilityPreferenceState = {
  highContrast: false,
  reducedMotion: false,
};

const SAVED_KEYS = ["highContrast", "reducedMotion", "schemaVersion"] as const;

function decodePreferences(raw: string): (AccessibilityPreferenceState & DisplayScaleState) | null {
  let value: unknown;
  try {
    value = JSON.parse(raw);
  } catch {
    return null;
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const record = value as Record<string, unknown>;
  const keys = Object.keys(record).sort();
  const expected = record.schemaVersion === 1 ? [...SAVED_KEYS] : record.schemaVersion === 2
    ? [...SAVED_KEYS, "textScalePercent", "uiScalePercent"].sort() : [];
  if (!expected.length || keys.length !== expected.length || !expected.every((key, index) => keys[index] === key)) return null;
  if (typeof record.highContrast !== "boolean" ||
      typeof record.reducedMotion !== "boolean") return null;
  if (record.schemaVersion === 2 && (!validScale("textScalePercent", record.textScalePercent) ||
      !validScale("uiScalePercent", record.uiScalePercent))) return null;
  return { highContrast: record.highContrast, reducedMotion: record.reducedMotion,
    textScalePercent: record.schemaVersion === 2 ? record.textScalePercent as number : 100,
    uiScalePercent: record.schemaVersion === 2 ? record.uiScalePercent as number : 100 };
}

function validScale(key: DisplayScaleKey, value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) &&
    (key === "textScalePercent" ? TEXT_SCALE_PERCENTAGES : UI_SCALE_PERCENTAGES).some(option => option === value);
}

/** Stores only local display preferences and mirrors them to the existing CSS/Pixi contract. */
export class AccessibilityPreferenceStore {
  private state: AccessibilityPreferenceState = { ...DEFAULT_STATE };
  private scale: DisplayScaleState = { ...DEFAULT_SCALE };
  private restored = false;
  private canPersist = false;

  constructor(
    private readonly getStorage: () => PreferenceStorage | null,
    private readonly target: PreferenceDatasetTarget,
  ) {}

  get(): AccessibilityPreferenceState {
    return { ...this.state };
  }

  restore(): AccessibilityPreferenceState {
    this.restored = true;
    let storage: PreferenceStorage | null;
    try {
      storage = this.getStorage();
    } catch {
      this.canPersist = false;
      return this.get();
    }
    if (!storage) {
      this.canPersist = false;
      return this.get();
    }

    let raw: string | null;
    try {
      raw = storage.getItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY);
    } catch {
      this.canPersist = false;
      return this.get();
    }
    if (raw === null) {
      this.canPersist = true;
      return this.get();
    }
    if (typeof raw !== "string") {
      this.canPersist = false;
      return this.get();
    }

    const saved = decodePreferences(raw);
    if (!saved) {
      // Keep the unknown/corrupt record intact; settings still work for this session.
      this.canPersist = false;
      return this.get();
    }
    this.state = { highContrast: saved.highContrast, reducedMotion: saved.reducedMotion };
    this.scale = { textScalePercent: saved.textScalePercent, uiScalePercent: saved.uiScalePercent };
    this.canPersist = true;
    this.apply();
    return this.get();
  }

  set(key: AccessibilityPreferenceKey, enabled: boolean): AccessibilityPreferenceState {
    if ((key !== "highContrast" && key !== "reducedMotion") || typeof enabled !== "boolean") return this.get();
    this.state = { ...this.state, [key]: enabled };
    this.apply();
    this.persist();
    return this.get();
  }

  getScale(): DisplayScaleState { return { ...this.scale }; }

  setScale(key: DisplayScaleKey, value: number): DisplayScaleState {
    if ((key !== "textScalePercent" && key !== "uiScalePercent") || !validScale(key, value)) return this.getScale();
    this.scale = { ...this.scale, [key]: value };
    this.apply();
    this.persist();
    return this.getScale();
  }

  private persist(): void {
    if (!this.restored || !this.canPersist) return;

    try {
      const storage = this.getStorage();
      if (!storage) {
        this.canPersist = false;
        return;
      }
      storage.setItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY, JSON.stringify({
        schemaVersion: 2,
        ...this.scale,
        highContrast: this.state.highContrast,
        reducedMotion: this.state.reducedMotion,
      }));
    } catch {
      // The in-memory state and UI remain usable even when local storage is blocked.
      this.canPersist = false;
    }
  }

  private apply(): void {
    if (this.scale.textScalePercent === 100) delete this.target.dataset.textScale;
    else this.target.dataset.textScale = String(this.scale.textScalePercent);
    if (this.scale.uiScalePercent === 100) delete this.target.dataset.uiScale;
    else this.target.dataset.uiScale = String(this.scale.uiScalePercent);
    if (this.state.highContrast) this.target.dataset.contrast = "high";
    else delete this.target.dataset.contrast;
    if (this.state.reducedMotion) this.target.dataset.motion = "reduced";
    else delete this.target.dataset.motion;
  }
}

/** Shares the same store semantics between settings buttons and startup restoration. */
export function bindAccessibilityPreferenceToggle(
  control: PreferenceToggleControl,
  preferences: AccessibilityPreferenceStore,
  key: AccessibilityPreferenceKey,
): void {
  const sync = () => control.setPressed(preferences.get()[key]);
  sync();
  control.onToggle(() => {
    preferences.set(key, !preferences.get()[key]);
    sync();
  });
}
