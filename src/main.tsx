import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { Palette } from "./Palette";
import { ColorMapsProvider } from "./lib/colors";
import { SettingsProvider } from "./lib/settings";
import "./index.css";

const label = getCurrentWindow().label;
const Component = label === "palette" ? Palette : App;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ColorMapsProvider>
      <SettingsProvider>
        <Component />
      </SettingsProvider>
    </ColorMapsProvider>
  </React.StrictMode>,
);
