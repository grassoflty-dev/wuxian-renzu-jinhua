import { isThemeSchema, type ThemeSchema } from "./ThemeSchema.js";
import { THEMES, WORLD_THEME_IDS } from "../themes/index.js";

export interface ThemeTarget {
  dataset: Record<string, string | undefined>;
  style: { setProperty(name: string, value: string): void };
}

export function resolveTheme(worldId: string, themes: Readonly<Record<string, ThemeSchema>> = THEMES): ThemeSchema {
  const themeId = WORLD_THEME_IDS[worldId] ?? "base";
  const theme = themes[themeId];
  return theme && isThemeSchema(theme) ? theme : THEMES.base;
}

export class ThemeManager {
  private currentThemeId = "";

  applyWorld(worldId: string, target: ThemeTarget): ThemeSchema {
    const theme = resolveTheme(worldId);
    if (!isThemeSchema(theme)) throw new Error("E_THEME_SCHEMA_INVALID");
    target.dataset.theme = theme.id;
    target.dataset.iconStyle = theme.iconStyle;
    target.dataset.mapStyle = theme.mapStyle;
    target.dataset.interactionStyle = theme.interactionStyle;
    for (const [key, value] of Object.entries(theme.colors)) {
      target.style.setProperty(`--theme-${key.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`, value);
    }
    target.style.setProperty("--theme-panel-radius", `${theme.panel.cornerRadius}px`);
    target.style.setProperty("--theme-border-width", `${theme.panel.borderWidth}px`);
    this.currentThemeId = theme.id;
    return theme;
  }

  get activeThemeId(): string { return this.currentThemeId; }
}
