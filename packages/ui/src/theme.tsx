import React, { createContext, useContext, useEffect, useState } from "react";
import type { ForgeTheme, ForgeDensity } from "@forge/tokens";

interface ThemeContextValue {
  theme: ForgeTheme;
  density: ForgeDensity;
  setTheme: (theme: ForgeTheme) => void;
  setDensity: (density: ForgeDensity) => void;
  toggleTheme: () => void;
  toggleDensity: () => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export interface ThemeProviderProps {
  defaultTheme?: ForgeTheme;
  defaultDensity?: ForgeDensity;
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({
  defaultTheme = "dark",
  defaultDensity = "compact",
  children,
}) => {
  const [theme, setTheme] = useState<ForgeTheme>(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("forge-theme") as ForgeTheme;
      if (saved && (saved === "dark" || saved === "light" || saved === "high-contrast")) {
        return saved;
      }
    }
    return defaultTheme;
  });

  const [density, setDensity] = useState<ForgeDensity>(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("forge-density") as ForgeDensity;
      if (saved && (saved === "compact" || saved === "comfortable")) {
        return saved;
      }
    }
    return defaultDensity;
  });

  useEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-theme", theme);
      localStorage.setItem("forge-theme", theme);
    }
  }, [theme]);

  useEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-density", density);
      localStorage.setItem("forge-density", density);
    }
  }, [density]);

  const toggleTheme = () => {
    setTheme((curr) => {
      if (curr === "dark") return "light";
      if (curr === "light") return "high-contrast";
      return "dark";
    });
  };

  const toggleDensity = () => {
    setDensity((curr) => (curr === "compact" ? "comfortable" : "compact"));
  };

  return (
    <ThemeContext.Provider
      value={{
        theme,
        density,
        setTheme,
        setDensity,
        toggleTheme,
        toggleDensity,
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
};

export const useTheme = (): ThemeContextValue => {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    return {
      theme: "dark",
      density: "compact",
      setTheme: () => {},
      setDensity: () => {},
      toggleTheme: () => {},
      toggleDensity: () => {},
    };
  }
  return ctx;
};
