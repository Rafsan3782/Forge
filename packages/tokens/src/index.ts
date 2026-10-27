/**
 * @forge/tokens
 * Design tokens for Forge IDE Studio and plugins.
 */

import tokensJson from "./tokens.json" with { type: "json" };

export interface ForgeTokens {
  theme: "dark";
  colors: {
    bg: {
      base: string;
      panel: string;
      surface: string;
      elevated: string;
      active: string;
      hover: string;
    };
    border: {
      base: string;
      subtle: string;
      focus: string;
    };
    text: {
      primary: string;
      muted: string;
      disabled: string;
      inverse: string;
    };
    accent: {
      ember: string;
      emberHover: string;
      arc: string;
      arcHover: string;
    };
    feedback: {
      success: string;
      warn: string;
      error: string;
    };
  };
  fonts: {
    sans: string;
    mono: string;
  };
  spacing: Record<string, string>;
  radius: Record<string, string>;
  motion: {
    duration: {
      fast: string;
      normal: string;
      slow: string;
    };
    easing: string;
  };
}

export const tokens: ForgeTokens = tokensJson as ForgeTokens;

export const colors = tokens.colors;
export const fonts = tokens.fonts;
export const spacing = tokens.spacing;
export const radius = tokens.radius;
export const motion = tokens.motion;

/**
 * CSS custom property names corresponding to @forge/tokens
 */
export const cssVars = {
  bgBase: "var(--forge-bg-base)",
  bgPanel: "var(--forge-bg-panel)",
  bgSurface: "var(--forge-bg-surface)",
  bgElevated: "var(--forge-bg-elevated)",
  bgActive: "var(--forge-bg-active)",
  bgHover: "var(--forge-bg-hover)",

  border: "var(--forge-border)",
  borderSubtle: "var(--forge-border-subtle)",
  borderFocus: "var(--forge-border-focus)",

  textPrimary: "var(--forge-text-primary)",
  textMuted: "var(--forge-text-muted)",
  textDisabled: "var(--forge-text-disabled)",
  textInverse: "var(--forge-text-inverse)",

  accentEmber: "var(--forge-accent-ember)",
  accentEmberHover: "var(--forge-accent-ember-hover)",
  accentArc: "var(--forge-accent-arc)",
  accentArcHover: "var(--forge-accent-arc-hover)",

  success: "var(--forge-success)",
  warn: "var(--forge-warn)",
  error: "var(--forge-error)",

  fontSans: "var(--forge-font-sans)",
  fontMono: "var(--forge-font-mono)",

  radiusXs: "var(--forge-radius-xs)",
  radiusSm: "var(--forge-radius-sm)",
  radiusMd: "var(--forge-radius-md)",
  radiusLg: "var(--forge-radius-lg)",
  radiusXl: "var(--forge-radius-xl)",
  radiusFull: "var(--forge-radius-full)",

  motionFast: "var(--forge-motion-fast)",
  motionNormal: "var(--forge-motion-normal)",
  motionSlow: "var(--forge-motion-slow)",
} as const;

export default tokens;
