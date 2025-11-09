import { createI18n } from 'vue-i18n'
import enUs from './locales/en-US.json'
import zhCn from './locales/zh-CN.json'
import zhTw from './locales/zh-TW.json'
import jaJp from './locales/ja-JP.json'
import frFr from './locales/fr-FR.json'

const i18n = createI18n({
  legacy: false,
  locale: navigator.language || 'en-US',
  fallbackLocale: 'en-US',
  messages: {
    'en-US': enUs,
    'zh-CN': zhCn,
    'zh-TW': zhTw,
    'ja-JP': jaJp,
    'fr-FR': frFr,
  },
  globalInjection: true,
})

export default i18n
