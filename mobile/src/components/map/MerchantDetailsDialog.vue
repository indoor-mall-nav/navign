<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { getMerchantDetails, type MerchantDetails } from '@/lib/api/tauri'
import { formatMerchantType } from '@/lib/structure/merchant'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { Icon } from '@iconify/vue'
import { error as logError } from '@tauri-apps/plugin-log'

const props = defineProps<{
  open: boolean
  entityId: string
  merchantId: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const merchantDetails = ref<MerchantDetails | null>(null)
const loading = ref(false)
const error = ref<string>('')

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen && props.merchantId) {
      await loadMerchantDetails()
    }
  },
)

async function loadMerchantDetails() {
  if (!props.merchantId) return

  loading.value = true
  error.value = ''

  try {
    const result = await getMerchantDetails(props.entityId, props.merchantId)
    if (result.status === 'success' && result.data) {
      merchantDetails.value = result.data
    } else {
      error.value = result.message || 'Failed to load merchant details'
    }
  } catch (err) {
    error.value = `Error: ${err}`
    await logError('Failed to load merchant details: ' + JSON.stringify(err))
  } finally {
    loading.value = false
  }
}

const merchantType = computed(() => {
  if (!merchantDetails.value?.type) return 'Unknown'
  try {
    return formatMerchantType(merchantDetails.value.type)
  } catch {
    return 'Unknown'
  }
})

const socialMediaIcons: Record<string, string> = {
  facebook: 'mdi:facebook',
  instagram: 'mdi:instagram',
  twitter: 'mdi:twitter',
  linkedin: 'mdi:linkedin',
  tiktok: 'simple-icons:tiktok',
  wechat: 'mdi:wechat',
  weibo: 'simple-icons:sinaweibo',
  rednote: 'mdi:notebook',
  bluesky: 'simple-icons:bluesky',
  reddit: 'mdi:reddit',
  discord: 'mdi:discord',
  whatsapp: 'mdi:whatsapp',
  telegram: 'mdi:telegram',
}

function getSocialIcon(platform: string): string {
  return socialMediaIcons[platform.toLowerCase()] || 'mdi:web'
}

function openLink(url: string) {
  window.open(url, '_blank')
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="max-w-2xl max-h-[80vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Icon icon="mdi:store" class="w-6 h-6 text-primary" />
          {{ merchantDetails?.name || 'Merchant Details' }}
        </DialogTitle>
        <DialogDescription
          v-if="merchantDetails?.chain"
          class="flex items-center gap-1"
        >
          <Icon icon="mdi:link-variant" class="w-4 h-4" />
          {{ merchantDetails.chain }}
        </DialogDescription>
      </DialogHeader>

      <div v-if="loading" class="space-y-4">
        <Skeleton class="h-20 w-full" />
        <Skeleton class="h-32 w-full" />
        <Skeleton class="h-24 w-full" />
      </div>

      <div v-else-if="error" class="text-center text-destructive py-8">
        <Icon icon="mdi:alert-circle" class="w-12 h-12 mx-auto mb-2" />
        {{ error }}
      </div>

      <div v-else-if="merchantDetails" class="space-y-4">
        <!-- Type & Style -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-center gap-4">
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                  <Icon
                    icon="mdi:tag-outline"
                    class="w-5 h-5 text-muted-foreground"
                  />
                  <h3 class="font-semibold">Type</h3>
                </div>
                <p class="text-sm text-muted-foreground">
                  {{ merchantType }}
                </p>
              </div>
              <div v-if="merchantDetails.style" class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                  <Icon
                    icon="mdi:shape-outline"
                    class="w-5 h-5 text-muted-foreground"
                  />
                  <h3 class="font-semibold">Style</h3>
                </div>
                <p class="text-sm text-muted-foreground capitalize">
                  {{ merchantDetails.style }}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Description -->
        <Card v-if="merchantDetails.description">
          <CardContent class="pt-6">
            <div class="flex items-start gap-3">
              <Icon
                icon="mdi:information-outline"
                class="w-5 h-5 text-muted-foreground mt-1"
              />
              <div>
                <h3 class="font-semibold mb-2">Description</h3>
                <p class="text-sm text-muted-foreground">
                  {{ merchantDetails.description }}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Tags -->
        <Card v-if="merchantDetails.tags.length > 0">
          <CardContent class="pt-6">
            <div class="flex items-start gap-3">
              <Icon
                icon="mdi:tag-multiple"
                class="w-5 h-5 text-muted-foreground mt-1"
              />
              <div class="flex-1">
                <h3 class="font-semibold mb-2">Tags</h3>
                <div class="flex flex-wrap gap-2">
                  <span
                    v-for="tag in merchantDetails.tags"
                    :key="tag"
                    class="text-xs bg-primary/10 text-primary px-2 py-1 rounded-full"
                  >
                    {{ tag }}
                  </span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Contact Information -->
        <Card
          v-if="
            merchantDetails.email ||
            merchantDetails.phone ||
            merchantDetails.website
          "
        >
          <CardContent class="pt-6">
            <div class="flex items-start gap-3">
              <Icon
                icon="mdi:contacts"
                class="w-5 h-5 text-muted-foreground mt-1"
              />
              <div class="flex-1 space-y-3">
                <h3 class="font-semibold mb-2">Contact</h3>

                <div
                  v-if="merchantDetails.email"
                  class="flex items-center gap-2"
                >
                  <Icon icon="mdi:email" class="w-4 h-4 text-muted-foreground" />
                  <a
                    :href="`mailto:${merchantDetails.email}`"
                    class="text-sm text-primary hover:underline"
                  >
                    {{ merchantDetails.email }}
                  </a>
                </div>

                <div
                  v-if="merchantDetails.phone"
                  class="flex items-center gap-2"
                >
                  <Icon icon="mdi:phone" class="w-4 h-4 text-muted-foreground" />
                  <a
                    :href="`tel:${merchantDetails.phone}`"
                    class="text-sm text-primary hover:underline"
                  >
                    {{ merchantDetails.phone }}
                  </a>
                </div>

                <div
                  v-if="merchantDetails.website"
                  class="flex items-center gap-2"
                >
                  <Icon icon="mdi:web" class="w-4 h-4 text-muted-foreground" />
                  <Button
                    variant="link"
                    size="sm"
                    class="h-auto p-0 text-sm"
                    @click="openLink(merchantDetails.website!)"
                  >
                    {{ merchantDetails.website }}
                  </Button>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Social Media -->
        <Card
          v-if="
            merchantDetails.social_media &&
            merchantDetails.social_media.length > 0
          "
        >
          <CardContent class="pt-6">
            <div class="flex items-start gap-3">
              <Icon
                icon="mdi:share-variant"
                class="w-5 h-5 text-muted-foreground mt-1"
              />
              <div class="flex-1">
                <h3 class="font-semibold mb-3">Social Media</h3>
                <div class="space-y-2">
                  <div
                    v-for="(social, idx) in merchantDetails.social_media"
                    :key="idx"
                    class="flex items-center gap-3"
                  >
                    <Icon
                      :icon="getSocialIcon(social.platform)"
                      class="w-5 h-5 text-muted-foreground"
                    />
                    <div class="flex-1">
                      <p class="text-sm font-medium capitalize">
                        {{ social.platform }}
                      </p>
                      <Button
                        v-if="social.url"
                        variant="link"
                        size="sm"
                        class="h-auto p-0 text-xs text-muted-foreground"
                        @click="openLink(social.url)"
                      >
                        {{ social.handle }}
                      </Button>
                      <p v-else class="text-xs text-muted-foreground">
                        {{ social.handle }}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Location -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-center gap-3">
              <Icon
                icon="mdi:map-marker"
                class="w-5 h-5 text-muted-foreground"
              />
              <div>
                <h3 class="font-semibold mb-1">Location</h3>
                <p class="text-sm text-muted-foreground">
                  ({{ merchantDetails.location[0].toFixed(2) }},
                  {{ merchantDetails.location[1].toFixed(2) }})
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Beacon Code -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-center gap-3">
              <Icon
                icon="mdi:access-point"
                class="w-5 h-5 text-muted-foreground"
              />
              <div>
                <h3 class="font-semibold mb-1">Beacon Code</h3>
                <p class="text-sm text-muted-foreground font-mono">
                  {{ merchantDetails.beacon_code }}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </DialogContent>
  </Dialog>
</template>
