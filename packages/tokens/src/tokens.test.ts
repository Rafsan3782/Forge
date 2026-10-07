import { describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { tokens, colors, fonts, cssVars } from "./index.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

describe("@forge/tokens", () => {
  it("exports required dark theme colors", () => {
    assert.equal(colors.bg.base, "#0A0B0D");
    assert.equal(colors.bg.panel, "#121418");
    assert.equal(colors.border.base, "#23262D");
    assert.equal(colors.text.primary, "#E8EAED");
    assert.equal(colors.text.muted, "#8B919C");
    assert.equal(colors.accent.ember, "#FF6A2B");
    assert.equal(colors.accent.arc, "#7C5CFF");
    assert.equal(colors.feedback.success, "#3DD68C");
    assert.equal(colors.feedback.warn, "#F5A524");
    assert.equal(colors.feedback.error, "#FF4D4F");
  });

  it("exports font families", () => {
    assert.ok(fonts.sans.includes("Geist Sans"));
    assert.ok(fonts.mono.includes("JetBrains Mono"));
  });

  it("exports matching CSS file", () => {
    const cssPath = path.join(__dirname, "tokens.css");
    assert.ok(fs.existsSync(cssPath), "tokens.css must exist");
    const content = fs.readFileSync(cssPath, "utf-8").toLowerCase();
    assert.ok(content.includes("--forge-bg-base: #0a0b0d;"));
    assert.ok(content.includes("--forge-accent-ember: #ff6a2b;"));
    assert.ok(content.includes("--forge-accent-arc: #7c5cff;"));
  });

  it("exports cssVars mapped to custom property syntax", () => {
    assert.equal(cssVars.bgBase, "var(--forge-bg-base)");
    assert.equal(cssVars.accentEmber, "var(--forge-accent-ember)");
    assert.equal(cssVars.accentArc, "var(--forge-accent-arc)");
  });

  it("exports JSON tokens", () => {
    const jsonPath = path.join(__dirname, "tokens.json");
    assert.ok(fs.existsSync(jsonPath), "tokens.json must exist");
    const raw = JSON.parse(fs.readFileSync(jsonPath, "utf-8"));
    assert.equal(raw.colors.bg.base, tokens.colors.bg.base);
  });
});
