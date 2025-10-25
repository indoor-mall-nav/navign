import { createI18n } from "vue-i18n";
import zhCn from "./locales/zh-CN.json";
import enUs from "./locales/en-US.json";

export default createI18n({
  locale: navigator.language || "en-US",
  fallbackLocale: "en-US",
  messages: {
    "zh-CN": zhCn,
    "en-US": enUs,
  } as Record<string, any>,
});
