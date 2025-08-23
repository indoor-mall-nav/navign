import { createApp } from "vue";
import App from "./App.vue";
import "./index.css";
import VConsole from "vconsole";
import VueKonva from "vue-konva";
import { createPinia } from "pinia";
import VueMaplibreGl from "vue-maplibre-gl";

new VConsole();

createApp(App)
  .use(createPinia())
  .use(VueMaplibreGl)
  .use(VueKonva)
  .mount("#app");
