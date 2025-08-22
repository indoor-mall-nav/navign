import { createApp } from "vue";
import App from "./App.vue";
import "./index.css";
import VConsole from "vconsole";
import VueKonva from 'vue-konva';
import { createPinia } from "pinia";

new VConsole();

createApp(App).use(createPinia()).use(VueKonva).mount("#app");
