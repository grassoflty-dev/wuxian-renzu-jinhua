import type { ThemeSchema } from "../theme/ThemeSchema.js";

const baseColors = {
  background: "#0B121A", panel: "#16202A", panelRaised: "#1B2934", border: "#344652",
  text: "#E7EDF2", textMuted: "#8D9AA5", system: "#58D7F0", accent: "#D9AD58",
  danger: "#E1504C", success: "#55C89C", focus: "#9FEEFF",
} as const;
const panel = { cornerRadius: 2, borderWidth: 1 } as const;

export const THEMES = {
  base: { id: "base", colors: baseColors, panel, iconStyle: "industrial", mapStyle: "station", interactionStyle: "terminal" },
  grey_hive: { id: "grey_hive", colors: baseColors, panel, iconStyle: "industrial", mapStyle: "facility", interactionStyle: "terminal" },
  mist_harbor: { id: "mist_harbor", colors: {
    ...baseColors, background: "#091721", panel: "#112936", panelRaised: "#1A3946", border: "#365866",
    textMuted: "#9AB4BE", system: "#82C9D4", accent: "#D8B879", focus: "#B2F3F5",
  }, panel, iconStyle: "nautical", mapStyle: "sonar", interactionStyle: "signal" },
  clockworks: { id: "clockworks", colors: {
    ...baseColors, background: "#17120E", panel: "#282018", panelRaised: "#382B1D", border: "#62503A",
    textMuted: "#B9A88D", system: "#D4A25D", accent: "#E08C45", danger: "#E1504C", focus: "#FFE0A1",
  }, panel, iconStyle: "brass", mapStyle: "mechanical", interactionStyle: "pressure" },
} satisfies Record<string, ThemeSchema>;

export const WORLD_THEME_IDS: Readonly<Record<string, keyof typeof THEMES>> = Object.freeze({
  return_station: "base",
  grey_hive: "grey_hive",
  mist_harbor: "mist_harbor",
  clockworks: "clockworks",
});
