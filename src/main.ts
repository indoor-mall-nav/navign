import { createApp } from "vue";
import App from "./App.vue";
import "./index.css";
import VConsole from "vconsole";
import VueKonva from "vue-konva";
import { createPinia } from "pinia";
import router from '@/router'
import i18n from '@/i18n'
import VueMaplibreGl from "vue-maplibre-gl";

new VConsole();

createApp(App)
  .use(createPinia())
  .use(VueMaplibreGl)
  .use(VueKonva)
  .use(router)
  .use(i18n)
  .mount("#app");
