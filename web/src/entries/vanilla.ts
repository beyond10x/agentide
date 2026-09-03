import "../styles.css";
import { mountFixtureHost } from "../host/fixture";
import { mountLocalHost } from "../host/local";
import { vanillaRenderer } from "../renderer/vanilla";

const container = document.querySelector<HTMLElement>("#app");
if (!container) throw new Error("missing #app");
if (new URLSearchParams(window.location.search).has("fixture")) {
  mountFixtureHost(vanillaRenderer, container);
} else {
  void mountLocalHost(vanillaRenderer, container);
}
