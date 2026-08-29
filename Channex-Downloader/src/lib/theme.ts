export type ThemePref = "system" | "light" | "dark";

const STORAGE_KEY = "channex-theme";

export function getStoredTheme(): ThemePref {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
}

export function applyTheme(pref: ThemePref) {
  const resolved = pref === "system" ? (systemPrefersDark() ? "dark" : "light") : pref;
  document.documentElement.dataset.theme = resolved;
}

export function storeTheme(pref: ThemePref) {
  localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(pref);
}

function systemPrefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function watchSystemTheme(getPref: () => ThemePref) {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const onChange = () => {
    if (getPref() === "system") applyTheme("system");
  };
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
}
