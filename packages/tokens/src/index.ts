/**
 * @forge/tokens
 * Design tokens for Forge IDE Studio and plugins (v2).
 */

import tokensJson from "./tokens.json" with { type: "json" };

export type ForgeTheme = "dark" | "light" | "high-contrast";
export type ForgeDensity = "compact" | "comfortable";

export interface ColorScale {
  surface: {
    base: string;
    raised: string;
    overlay: string;
    sunken: string;
    active: string;
    hover: string;
  };
  border: {
    subtle: string;
    base: string;
    strong: string;
    focus: string;
  };
  text: {
    primary: string;
    muted: string;
    disabled: string;
    inverse: string;
  };
  accent: {
    base: string;
    hover: string;
    active: string;
    subtle: string;
    contrast: string;
  };
  ai: {
    base: string;
    hover: string;
    active: string;
    subtle: string;
    contrast: string;
  };
  feedback: {
    success: string;
    successSubtle: string;
    warn: string;
    warnSubtle: string;
    danger: string;
    dangerSubtle: string;
    info: string;
    infoSubtle: string;
  };
  selection: {
    bg: string;
    border: string;
  };
  syntax: {
    keyword: string;
    entity: string;
    string: string;
    number: string;
    comment: string;
    variable: string;
    constant: string;
    type: string;
  };
}

export interface ForgeTokensV2 {
  name: string;
  version: string;
  themes: Record<ForgeTheme, { colors: ColorScale }>;
  fonts: {
    sans: string;
    mono: string;
  };
  typography: {
    size: Record<string, string>;
    lineHeight: Record<string, string>;
    weight: Record<string, string>;
  };
  radius: Record<string, string>;
  spacing: Record<string, string>;
  controlHeight: Record<ForgeDensity, Record<string, string>>;
  elevation: Record<string, string>;
  motion: {
    duration: Record<string, string>;
    easing: Record<string, string>;
  };
  zIndex: Record<string, number>;
}

export const tokens = tokensJson as unknown as ForgeTokensV2;

// Backwards-compatible legacy colors accessor for dark theme
export const colors = {
  bg: {
    base: tokens.themes.dark.colors.surface.base,
    panel: tokens.themes.dark.colors.surface.raised,
    surface: tokens.themes.dark.colors.surface.overlay,
    elevated: tokens.themes.dark.colors.surface.overlay,
    active: tokens.themes.dark.colors.surface.active,
    hover: tokens.themes.dark.colors.surface.hover,
  },
  border: {
    base: tokens.themes.dark.colors.border.base,
    subtle: tokens.themes.dark.colors.border.subtle,
    focus: tokens.themes.dark.colors.border.focus,
  },
  text: {
    primary: tokens.themes.dark.colors.text.primary,
    muted: tokens.themes.dark.colors.text.muted,
    disabled: tokens.themes.dark.colors.text.disabled,
    inverse: tokens.themes.dark.colors.text.inverse,
  },
  accent: {
    ember: tokens.themes.dark.colors.accent.base,
    emberHover: tokens.themes.dark.colors.accent.hover,
    arc: tokens.themes.dark.colors.ai.base,
    arcHover: tokens.themes.dark.colors.ai.hover,
  },
  feedback: {
    success: tokens.themes.dark.colors.feedback.success,
    warn: tokens.themes.dark.colors.feedback.warn,
    error: tokens.themes.dark.colors.feedback.danger,
  },
};

export const fonts = tokens.fonts;
export const spacing = tokens.spacing;
export const radius = tokens.radius;
export const motion = tokens.motion;
export const typography = tokens.typography;
export const controlHeight = tokens.controlHeight;
export const elevation = tokens.elevation;
export const zIndex = tokens.zIndex;

/**
 * CSS custom property names corresponding to @forge/tokens
 */
export const cssVars = {
  // Surfaces
  surfaceBase: "var(--forge-surface-base)",
  surfaceRaised: "var(--forge-surface-raised)",
  surfaceOverlay: "var(--forge-surface-overlay)",
  surfaceSunken: "var(--forge-surface-sunken)",
  surfaceActive: "var(--forge-surface-active)",
  surfaceHover: "var(--forge-surface-hover)",

  // Legacy bg names
  bgBase: "var(--forge-bg-base)",
  bgPanel: "var(--forge-bg-panel)",
  bgSurface: "var(--forge-bg-surface)",
  bgElevated: "var(--forge-bg-elevated)",
  bgActive: "var(--forge-bg-active)",
  bgHover: "var(--forge-bg-hover)",

  // Borders
  border: "var(--forge-border)",
  borderSubtle: "var(--forge-border-subtle)",
  borderStrong: "var(--forge-border-strong)",
  borderFocus: "var(--forge-border-focus)",

  // Typography Colors
  textPrimary: "var(--forge-text-primary)",
  textMuted: "var(--forge-text-muted)",
  textDisabled: "var(--forge-text-disabled)",
  textInverse: "var(--forge-text-inverse)",

  // Accents
  accent: "var(--forge-accent)",
  accentHover: "var(--forge-accent-hover)",
  accentActive: "var(--forge-accent-active)",
  accentSubtle: "var(--forge-accent-subtle)",
  accentContrast: "var(--forge-accent-contrast)",

  // Legacy accent names
  accentEmber: "var(--forge-accent-ember)",
  accentEmberHover: "var(--forge-accent-ember-hover)",

  // AI Accents
  ai: "var(--forge-ai)",
  aiHover: "var(--forge-ai-hover)",
  aiActive: "var(--forge-ai-active)",
  aiSubtle: "var(--forge-ai-subtle)",
  aiContrast: "var(--forge-ai-contrast)",

  // Legacy AI names
  accentArc: "var(--forge-accent-arc)",
  accentArcHover: "var(--forge-accent-arc-hover)",

  // Feedback
  success: "var(--forge-success)",
  warn: "var(--forge-warn)",
  danger: "var(--forge-danger)",
  error: "var(--forge-error)",
  info: "var(--forge-info)",

  // Selection
  selectionBg: "var(--forge-selection-bg)",
  selectionBorder: "var(--forge-selection-border)",
  focusRing: "var(--forge-focus-ring)",

  // Fonts
  fontSans: "var(--forge-font-sans)",
  fontMono: "var(--forge-font-mono)",

  // Radii
  radiusNone: "var(--forge-radius-none)",
  radiusXs: "var(--forge-radius-xs)",
  radiusSm: "var(--forge-radius-sm)",
  radiusMd: "var(--forge-radius-md)",
  radiusLg: "var(--forge-radius-lg)",
  radiusXl: "var(--forge-radius-xl)",
  radiusFull: "var(--forge-radius-full)",

  // Spacing
  space0: "var(--forge-space-0)",
  space1: "var(--forge-space-1)",
  space2: "var(--forge-space-2)",
  space3: "var(--forge-space-3)",
  space4: "var(--forge-space-4)",
  space5: "var(--forge-space-5)",
  space6: "var(--forge-space-6)",
  space8: "var(--forge-space-8)",
  space10: "var(--forge-space-10)",
  space12: "var(--forge-space-12)",

  // Control Heights
  controlHeightSm: "var(--forge-control-height-sm)",
  controlHeightMd: "var(--forge-control-height-md)",
  controlHeightLg: "var(--forge-control-height-lg)",

  // Motion
  motionFast: "var(--forge-motion-fast)",
  motionNormal: "var(--forge-motion-normal)",
  motionSlow: "var(--forge-motion-slow)",

  // Elevation
  elevationLow: "var(--forge-elevation-low)",
  elevationMedium: "var(--forge-elevation-medium)",
  elevationHigh: "var(--forge-elevation-high)",
  elevationFloating: "var(--forge-elevation-floating)",
} as const;

export * from "./tailwind.ts";
export default tokens;
