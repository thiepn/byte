import { mount } from "svelte";
import App from "./app/App.svelte";
import "./styles/tokens.css";
import "./styles/themes.css";
import "./styles/global.css";

mount(App, {
  target: document.getElementById("app")!,
});
