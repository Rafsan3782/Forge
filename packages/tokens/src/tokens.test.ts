import { describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { tokens, colors, fonts, cssVars, forgeTailwindPreset } from "./index.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Simple sRGB luminance calculation
function getLuminance(hex: string): number {
  let c = hex.replace("#", "");
  if (c.length === 3) {
    c = c.split("").map((x) => x + x).join("");
  }
  const num = parseInt(c, 16);
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;

  const a = [r, g, b].map((v) => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
}

function getContrastRatio(hex1: string, hex2: string): number {
  const l1 = getLuminance(hex1);
  const l2 = getLuminance(hex2);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

describe("@forge/tokens v2", () => {
  it("exports required dark theme colors", () => {
    assert.equal(colors.bg.base.toLowerCase(), "#0a0b0e");
    assert.equal(colors.bg.panel.toLowerCase(), "#12141a");
    assert.equal(colors.border.base.toLowerCase(), "#282d39");
    assert.equal(colors.text.primary.toLowerCase(), "#f0f2f5");
    assert.equal(colors.text.muted.toLowerCase(), "#8d95a5");
    assert.equal(colors.accent.ember.toLowerCase(), "#ff5c26");
    assert.equal(colors.accent.arc.toLowerCase(), "#8b5cf6");
    assert.equal(colors.feedback.success.toLowerCase(), "#22c55e");
    assert.equal(colors.feedback.warn.toLowerCase(), "#f59e0b");
    assert.equal(colors.feedback.error.toLowerCase(), "#ef4444");
  });

  it("exports all 3 required themes (dark, light, high-contrast)", () => {
    assert.ok(tokens.themes["dark"]);
    assert.ok(tokens.themes["light"]);
    assert.ok(tokens.themes["high-contrast"]);
  });

  it("exports typography and 4px spacing scale", () => {
    assert.equal(tokens.spacing["1"], "4px");
    assert.equal(tokens.spacing["2"], "8px");
    assert.equal(tokens.spacing["3"], "12px");
    assert.equal(tokens.spacing["4"], "16px");

    assert.equal(tokens.typography.size.xs, "11px");
    assert.equal(tokens.typography.size.sm, "12px");
    assert.equal(tokens.typography.size.base, "13px");
    assert.equal(tokens.typography.size.md, "14px");
  });

  it("exports control heights for compact and comfortable densities", () => {
    assert.equal(tokens.controlHeight.compact.sm, "22px");
    assert.equal(tokens.controlHeight.compact.md, "26px");
    assert.equal(tokens.controlHeight.compact.lg, "30px");

    assert.equal(tokens.controlHeight.comfortable.sm, "26px");
    assert.equal(tokens.controlHeight.comfortable.md, "30px");
    assert.equal(tokens.controlHeight.comfortable.lg, "34px");
  });

  it("passes WCAG AA contrast ratio (>= 4.5:1) for primary text on surface backgrounds", () => {
    // Dark theme
    const darkRatio = getContrastRatio(
      tokens.themes.dark.colors.text.primary,
      tokens.themes.dark.colors.surface.base
    );
    assert.ok(
      darkRatio >= 4.5,
      `Dark theme text contrast ratio ${darkRatio} must be >= 4.5:1`
    );

    // Light theme
    const lightRatio = getContrastRatio(
      tokens.themes.light.colors.text.primary,
      tokens.themes.light.colors.surface.base
    );
    assert.ok(
      lightRatio >= 4.5,
      `Light theme text contrast ratio ${lightRatio} must be >= 4.5:1`
    );

    // High contrast theme
    const hcRatio = getContrastRatio(
      tokens.themes["high-contrast"].colors.text.primary,
      tokens.themes["high-contrast"].colors.surface.base
    );
    assert.ok(
      hcRatio >= 7.0,
      `High-contrast text ratio ${hcRatio} must be >= 7.0:1 (AAA)`
    );
  });

  it("exports Tailwind preset", () => {
    assert.ok(forgeTailwindPreset.theme.extend.colors.forge.surface.base);
  });

  it("exports matching CSS file", () => {
    const cssPath = path.join(__dirname, "tokens.css");
    assert.ok(fs.existsSync(cssPath), "tokens.css must exist");
    const content = fs.readFileSync(cssPath, "utf-8").toLowerCase();
    assert.ok(content.includes("--forge-surface-base: #0a0b0e;"));
    assert.ok(content.includes("--forge-accent: #ff5c26;"));
    assert.ok(content.includes("--forge-ai: #8b5cf6;"));
    assert.ok(content.includes('[data-theme="light"]'));
    assert.ok(content.includes('[data-theme="high-contrast"]'));
    assert.ok(content.includes('[data-density="compact"]'));
    assert.ok(content.includes('[data-density="comfortable"]'));
  });

  it("exports JSON tokens", () => {
    const jsonPath = path.join(__dirname, "tokens.json");
    assert.ok(fs.existsSync(jsonPath), "tokens.json must exist");
    const raw = JSON.parse(fs.readFileSync(jsonPath, "utf-8"));
    assert.equal(raw.themes.dark.colors.surface.base, tokens.themes.dark.colors.surface.base);
  });
});
