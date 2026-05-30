import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { Palette } from "./Palette";
import { Onboarding } from "./Onboarding";
import { ColorMapsProvider } from "./lib/colors";
import { SettingsProvider } from "./lib/settings";
import "./index.css";

const label = getCurrentWindow().label;

let tree: React.ReactNode;
if (label === "onboarding") {
  // Onboarding runs before AppState is managed in Rust — SettingsProvider /
  // ColorMapsProvider would only spam errors from `invoke("get_settings")`.
  // Skip them; the Onboarding component is fully self-contained.
  tree = <Onboarding />;
} else {
  const Component = label === "palette" ? Palette : App;
  tree = (
    <ColorMapsProvider>
      <SettingsProvider>
        <Component />
      </SettingsProvider>
    </ColorMapsProvider>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{tree}</React.StrictMode>,
);
