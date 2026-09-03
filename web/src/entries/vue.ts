import "../styles.css";
import { mountFixtureHost } from "../host/fixture";
import { mountLocalHost } from "../host/local";
import { vueRenderer } from "../renderer/vue";

const container = document.querySelector<HTMLElement>("#app");
if (!container) throw new Error("missing #app");
if (new URLSearchParams(window.location.search).has("fixture")) {
  mountFixtureHost(vueRenderer, container);
} else {
  void mountLocalHost(vueRenderer, container);
}
