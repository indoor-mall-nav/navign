<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Icon } from '@iconify/vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'

const { locale } = useI18n()

interface Language {
  code: string
  name: string
  nativeName: string
  flag: string
}

const languages: Language[] = [
  {
    code: 'en-US',
    name: 'English',
    nativeName: 'English',
    flag: '🇺🇸',
  },
  {
    code: 'zh-CN',
    name: 'Simplified Chinese',
    nativeName: '简体中文',
    flag: '🇨🇳',
  },
  {
    code: 'zh-TW',
    name: 'Traditional Chinese',
    nativeName: '繁體中文',
    flag: '🇹🇼',
  },
  {
    code: 'ja-JP',
    name: 'Japanese',
    nativeName: '日本語',
    flag: '🇯🇵',
  },
  {
    code: 'fr-FR',
    name: 'French',
    nativeName: 'Français',
    flag: '🇫🇷',
  },
]

const currentLanguage = computed(() => {
  return languages.find((lang) => lang.code === locale.value) || languages[0]
})

function changeLanguage(languageCode: string) {
  locale.value = languageCode
  // Store preference in localStorage
  localStorage.setItem('preferred_language', languageCode)
}
</script>

<template>
  <Dialog>
    <DialogTrigger as-child>
      <Button variant="outline" size="sm" class="gap-2">
        <span class="text-lg">{{ currentLanguage.flag }}</span>
        <span class="hidden sm:inline">{{ currentLanguage.nativeName }}</span>
        <Icon icon="mdi:chevron-down" class="w-4 h-4" />
      </Button>
    </DialogTrigger>
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Icon icon="mdi:translate" class="w-5 h-5" />
          Select Language
        </DialogTitle>
        <DialogDescription>
          Choose your preferred language for the application
        </DialogDescription>
      </DialogHeader>
      <div class="grid gap-2 py-4">
        <Card
          v-for="language in languages"
          :key="language.code"
          class="p-3 cursor-pointer transition-colors hover:bg-accent"
          :class="{
            'bg-accent border-primary': locale === language.code,
          }"
          @click="changeLanguage(language.code)"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span class="text-2xl">{{ language.flag }}</span>
              <div>
                <p class="font-medium">{{ language.nativeName }}</p>
                <p class="text-sm text-muted-foreground">{{ language.name }}</p>
              </div>
            </div>
            <Icon
              v-if="locale === language.code"
              icon="mdi:check-circle"
              class="w-5 h-5 text-primary"
            />
          </div>
        </Card>
      </div>
    </DialogContent>
  </Dialog>
</template>
