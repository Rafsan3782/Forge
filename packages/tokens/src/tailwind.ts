/**
 * @forge/tokens/tailwind
 * Tailwind CSS v3 / v4 compatible preset for Forge tokens v2
 */

export const forgeTailwindPreset = {
  theme: {
    extend: {
      colors: {
        forge: {
          surface: {
            base: "var(--forge-surface-base)",
            raised: "var(--forge-surface-raised)",
            overlay: "var(--forge-surface-overlay)",
            sunken: "var(--forge-surface-sunken)",
            active: "var(--forge-surface-active)",
            hover: "var(--forge-surface-hover)",
          },
          border: {
            subtle: "var(--forge-border-subtle)",
            DEFAULT: "var(--forge-border)",
            strong: "var(--forge-border-strong)",
            focus: "var(--forge-border-focus)",
          },
          text: {
            primary: "var(--forge-text-primary)",
            muted: "var(--forge-text-muted)",
            disabled: "var(--forge-text-disabled)",
            inverse: "var(--forge-text-inverse)",
          },
          accent: {
            DEFAULT: "var(--forge-accent)",
            hover: "var(--forge-accent-hover)",
            active: "var(--forge-accent-active)",
            subtle: "var(--forge-accent-subtle)",
            contrast: "var(--forge-accent-contrast)",
            ember: "var(--forge-accent)",
          },
          ai: {
            DEFAULT: "var(--forge-ai)",
            hover: "var(--forge-ai-hover)",
            active: "var(--forge-ai-active)",
            subtle: "var(--forge-ai-subtle)",
            contrast: "var(--forge-ai-contrast)",
            arc: "var(--forge-ai)",
          },
          feedback: {
            success: "var(--forge-success)",
            warn: "var(--forge-warn)",
            danger: "var(--forge-danger)",
            info: "var(--forge-info)",
          },
        },
      },
      fontFamily: {
        sans: ["var(--forge-font-sans)"],
        mono: ["var(--forge-font-mono)"],
      },
      borderRadius: {
        xs: "var(--forge-radius-xs)",
        sm: "var(--forge-radius-sm)",
        md: "var(--forge-radius-md)",
        lg: "var(--forge-radius-lg)",
        xl: "var(--forge-radius-xl)",
      },
      spacing: {
        "4px": "4px",
        "8px": "8px",
        "12px": "12px",
        "16px": "16px",
        "20px": "20px",
        "24px": "24px",
      },
    },
  },
};

export default forgeTailwindPreset;
