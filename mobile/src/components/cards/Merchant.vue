<script setup lang="ts">
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Merchant } from '@/schema'
import { Icon } from '@iconify/vue'
import { toRefs } from 'vue'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { openUrl } from '@tauri-apps/plugin-opener'
import { formatMerchantType } from '@/lib/structure/merchant.ts'
import { info } from '@tauri-apps/plugin-log'

const props = defineProps<{
  merchant: Merchant
}>()

const { merchant } = toRefs(props)

function generateSocialMediaUrl(media: Merchant['social_media'][0]): {
  type: 'url' | 'qrcode'
  url?: string
} {
  const handle = media.handle.replace(/^@/, '') // Remove leading '@' if present
  if (!handle) {
    return { type: 'url', url: '#' } // Fallback to direct URL if handle is empty
  }
  if (media.url) {
    return { type: 'url', url: media.url }
  }
  switch (media.platform) {
    case 'twitter':
      return { type: 'url', url: `https://twitter.com/${handle}` }
    case 'facebook':
      return { type: 'url', url: `https://facebook.com/${handle}` }
    case 'instagram':
      return { type: 'url', url: `https://instagram.com/${handle}` }
    case 'linkedin':
      return { type: 'url', url: `https://linkedin.com/in/${handle}` }
    case 'wechat':
      return { type: 'qrcode', url: handle } // WeChat typically uses QR codes
    case 'whatsapp':
      return { type: 'url', url: `https://wa.me/${handle}` }
    case 'telegram':
      return { type: 'url', url: `https://t.me/${handle}` }
    default:
      return { type: 'url', url: '#' } // Fallback to direct URL
  }
}

// The `opening_hours` is a (start, end) milliseconds tuple
// representing the opening hours of the merchant in a day.
function processOpeningHours(opening_hours: [number, number] | []): string {
  if (opening_hours.length === 0) {
    return 'Closed'
  }
  if (opening_hours[0] === 0 && opening_hours[1] === 86400000) {
    return 'Open 24 hours'
  }
  const [start, end] = opening_hours
  const startDate = new Date(start)
  const endDate = new Date(end)
  const options: Intl.DateTimeFormatOptions = {
    hour: '2-digit',
    minute: '2-digit',
  }
  return `${startDate.toLocaleTimeString([], options)} – ${endDate.toLocaleTimeString([], options)}`
}

function processTodayOpeningHours(
  opening_hours: ([number, number] | [])[],
): string {
  const today = new Date()
  const dayOfWeek = today.getDay() // 0 (Sunday) to 6 (Saturday)
  if (dayOfWeek < opening_hours.length) {
    return processOpeningHours(opening_hours[dayOfWeek])
  }
  return 'Closed'
}
</script>

<template>
  <Card class="my-2">
    <CardHeader>
      <CardTitle class="text-lg">{{ merchant.name }}</CardTitle>
      <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
        {{ merchant.description }}
      </p>
    </CardHeader>
    <CardContent>
      <p class="flex">
        <span class="font-medium">
          {{ formatMerchantType(merchant.type) }}
        </span>
        <Separator orientation="vertical" class="mx-1" />
        <!-- if during operation hours, show green dot, else red dot -->
        <span class="font-medium">
          <span
            v-if="
              new Date().getHours() >=
                (merchant.opening_hours[new Date().getDay()]?.[0] ?? 86400000) /
                  3600000 &&
              new Date().getHours() <
                (merchant.opening_hours[new Date().getDay()]?.[1] ?? 0) /
                  3600000
            "
            class="text-green-600 dark:text-green-400"
          >
            Open
          </span>
          <span v-else class="text-red-600 dark:text-red-400">
            Currently Closed, Today:
            {{ processTodayOpeningHours(merchant.opening_hours) }}
          </span>
        </span>
      </p>
    </CardContent>
    <CardAction>
      <div class="flex mx-4">
        <Button variant="outline" class="mx-2">
          <Icon icon="tabler:map-pin" class="w-5 h-5 mr-2" />
          Goto
        </Button>
        <Button
          v-if="merchant.email"
          variant="link"
          size="icon"
          class="mb-2"
          @click="openUrl(`mailto:${merchant.email}`)"
        >
          <Icon icon="tabler:mail" class="w-5 h-5 mr-2" />
        </Button>
        <Button
          v-if="merchant.phone"
          variant="link"
          size="icon"
          class="mb-2"
          @click="openUrl(`tel:${merchant.phone}`)"
        >
          <Icon icon="tabler:phone" class="w-5 h-5 mr-2" />
        </Button>
        <Button
          v-if="merchant.website"
          variant="link"
          size="icon"
          class="mb-2"
          @click="openUrl(merchant.website)"
        >
          <Icon icon="tabler:world" class="w-5 h-5 mr-2" />
        </Button>
        <Button
          v-for="(social, index) in merchant.social_media.map((x) => ({
            ...x,
            ...generateSocialMediaUrl(x),
          }))"
          :key="index"
          variant="link"
          size="icon"
          class="mb-2"
          @click="
            social.type === 'url'
              ? openUrl(social.url ?? '#')
              : info(`WeChat QR Code: ${social.url}`)
          "
        >
          <Icon
            :icon="`tabler:brand-${social.platform}`"
            class="w-5 h-5 mr-2"
          />
        </Button>
      </div>
    </CardAction>
  </Card>
</template>

<style scoped></style>
