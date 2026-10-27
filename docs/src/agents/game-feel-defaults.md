---
id: agents.game-feel-defaults
title: Game Feel Defaults and Tuning Tables
summary: Curated reference values for platformers, top-down games, and puzzle games.
tags: ["agents", "game-feel", "tuning", "physics", "camera"]
audience: agent
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "agents.playbooks"]
---

# Game Feel Defaults & Tuning Tables

Use these verified numbers as default parameters when generating or tuning games.

## 1. 2D Platformer

| Parameter | Recommended Value | Unit | Impact on Game Feel |
| :--- | :--- | :--- | :--- |
| **Run Speed** | `220.0` | `px/s` | Responsive horizontal movement without feeling slippery. |
| **Acceleration** | `1800.0` | `px/s²` | Near-instant ground responsiveness (~0.12s to max speed). |
| **Deceleration / Friction** | `2400.0` | `px/s²` | Clean stops without excessive skidding. |
| **Gravity** | `980.0` | `px/s²` | Weighty, snappy downward acceleration. |
| **Jump Velocity** | `440.0` | `px/s` | Jump height ~98 pixels (~3 standard 32px tiles). |
| **Jump Time to Peak** | `0.45` | `s` | Duration from ground takeoff to highest point. |
| **Max Fall Speed (Terminal)** | `550.0` | `px/s` | Prevents falling through thin colliders. |
| **Coyote Time** | `0.12` | `s` | Forgiving jump window after walking off platform edges. |
| **Jump Buffer Time** | `0.12` | `s` | Registers jump input 7-8 frames before touching ground. |
| **Variable Jump Fall Mult** | `2.2` | scalar | Gravity multiplier when jump button is released early. |
| **Camera Smooth Speed** | `5.0` | scalar | Smooth tracking with 0.1-0.2s lag behind hero. |
| **Screen Shake Duration** | `0.18` | `s` | Snappy impact feel on hurt or explosion. |
| **Screen Shake Amplitude** | `6.0` | `px` | Noticeable camera trauma without disorienting player. |
| **Animation FPS** | `8` to `12` | `frames/s` | Pixel-art retro animation cadence. |

---

## 2. 2D Top-Down Action / Adventure

| Parameter | Recommended Value | Unit | Impact on Game Feel |
| :--- | :--- | :--- | :--- |
| **Movement Speed** | `180.0` | `px/s` | Balanced speed for directional exploration and combat. |
| **Diagonal Normalization** | `0.7071` (1/√2) | scalar | Prevents diagonal movement from being ~41% faster. |
| **Acceleration** | `1200.0` | `px/s²` | Smooth momentum ramp for top-down hero. |
| **Friction** | `1600.0` | `px/s²` | Tight stop after releasing WASD / stick. |
| **Dash Speed** | `450.0` | `px/s` | High-speed dodge mechanic. |
| **Dash Duration** | `0.15` | `s` | Fast burst through hazards or enemies. |
| **Dash Cooldown** | `0.6` | `s` | Prevents spamming dash across map. |
| **Camera Deadzone** | `[40.0, 30.0]` | `px` | Small center box where camera does not move. |
| **Camera Lead / Lookahead** | `30.0` | `px` | Nudges camera forward in direction of travel. |
| **Bullet Speed** | `400.0` | `px/s` | Projectile readable by human eye. |

---

## 3. 2D Puzzle / Grid Movement

| Parameter | Recommended Value | Unit | Impact on Game Feel |
| :--- | :--- | :--- | :--- |
| **Grid Cell Size** | `32.0` | `px` | Standard tile granularity. |
| **Move Duration per Cell** | `0.14` | `s` | Snappy step transition between adjacent tiles. |
| **Move Easing** | EaseOutQuad | function | Starts fast and settles smoothly onto tile grid. |
| **Input Repeat Delay** | `0.25` | `s` | Delay before continuous directional hold repeats step. |
| **Input Repeat Rate** | `0.10` | `s` | Speed of successive grid steps while holding direction. |
| **Undo Stack Depth** | `100` | steps | Sufficient history for rewinding puzzle mistakes. |
| **Turn Transition Pause** | `0.05` | `s` | Micro-pause to allow state change comprehension. |
