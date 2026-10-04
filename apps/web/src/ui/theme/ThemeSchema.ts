export interface ThemeSchema {
  id: string;
  colors: {
    background: string;
    panel: string;
    panelRaised: string;
    border: string;
    text: string;
    textMuted: string;
    system: string;
    accent: string;
    danger: string;
    success: string;
    focus: string;
  };
  panel: {
    cornerRadius: number;
    borderWidth: number;
  };
  iconStyle: string;
  mapStyle: string;
  interactionStyle: string;
}

const COLOR_KEYS = [
  "background", "panel", "panelRaised", "border", "text", "textMuted", "system", "accent", "danger", "success", "focus",
] as const;
const HEX_COLOR = /^#[\da-f]{6}$/i;

export function isThemeSchema(value: unknown): value is ThemeSchema {
  if (!value || typeof value !== "object") return false;
  const theme = value as Partial<ThemeSchema>;
  if (typeof theme.id !== "string" || !/^[a-z][a-z0-9_]*$/.test(theme.id)) return false;
  if (!theme.colors || typeof theme.colors !== "object" || !COLOR_KEYS.every(key =>
    typeof theme.colors?.[key] === "string" && HEX_COLOR.test(theme.colors[key]))) return false;
  if (!theme.panel || typeof theme.panel !== "object" || !Number.isFinite(theme.panel.cornerRadius) ||
      theme.panel.cornerRadius < 0 || !Number.isFinite(theme.panel.borderWidth) || theme.panel.borderWidth < 0) return false;
  return [theme.iconStyle, theme.mapStyle, theme.interactionStyle].every(style =>
    typeof style === "string" && /^[a-z][a-z0-9_-]*$/.test(style));
}
