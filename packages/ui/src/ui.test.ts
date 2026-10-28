import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { PACKAGE_NAME } from "./index.js";
import { CustomIcons } from "./Icon.js";

describe("@forge/ui v2", () => {
  it("exports package name and tokens", () => {
    assert.equal(PACKAGE_NAME, "@forge/ui");
  });

  it("exports all engine custom icons", () => {
    const requiredIcons = [
      "entity",
      "component",
      "scene",
      "tilemap",
      "sprite",
      "collider",
      "script",
      "play",
      "stop",
      "step",
      "gizmo-move",
      "gizmo-rotate",
      "gizmo-scale",
      "checkpoint",
      "agent",
    ];

    for (const icon of requiredIcons) {
      assert.ok(icon in CustomIcons, `Icon ${icon} must be present in CustomIcons`);
    }
  });
});
