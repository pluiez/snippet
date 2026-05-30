import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { Palette } from "./Palette";
import { Onboarding } from "./Onboarding";
import { ColorMapsProvider } from "./lib/colors";
import { SettingsProvider, useSettings } from "./lib/settings";
import { ThemeApplier } from "./lib/theme";
import "./index.css";

/** Bridge: reads the theme preference from SettingsProvider context. */
function SettingsThemeApplier() {
  const { settings } = useSettings();
  return <ThemeApplier theme={settings?.theme ?? null} />;
}

const label = getCurrentWindow().label;

let tree: React.ReactNode;
if (label === "onboarding") {
  // Onboarding runs before AppState is managed in Rust — SettingsProvider /
  // ColorMapsProvider would only spam errors from `invoke("get_settings")`.
  // Skip them; the Onboarding component is fully self-contained.
  // Theme defaults to "system" (no settings exist yet during onboarding).
  tree = (
    <>
      <ThemeApplier theme="system" />
      <Onboarding />
    </>
  );
} else {
  const Component = label === "palette" ? Palette : App;
  tree = (
    <ColorMapsProvider>
      <SettingsProvider>
        <SettingsThemeApplier />
        <Component />
      </SettingsProvider>
    </ColorMapsProvider>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{tree}</React.StrictMode>,
);
