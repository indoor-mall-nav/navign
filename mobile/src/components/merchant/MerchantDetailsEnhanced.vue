<script setup lang="ts">
import { computed, ref } from "vue";
import type { Merchant } from "@/schema";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@iconify/vue";
import { useFavoritesStore } from "@/states/favorites";

interface Props {
  merchant: Merchant;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  navigate: [];
  close: [];
}>();

const favorites = useFavoritesStore();
const activeTab = ref<"details" | "contact" | "hours">("details");

const isFavorited = computed(() => {
  const merchantId = props.merchant.id || "";
  return favorites.isMerchantFavorited(merchantId);
});

const merchantTypeLabel = computed(() => {
  const type = props.merchant.type;
  if (typeof type === "string") {
    return type;
  } else if ("food" in type) {
    return "Food";
  } else if ("electronics" in type) {
    return "Electronics";
  } else if ("clothing" in type) {
    return "Clothing";
  }
  return "Other";
});

function toggleFavorite() {
  const merchantId = props.merchant.id || "";
  if (isFavorited.value) {
    favorites.removeMerchantFavorite(merchantId);
  } else {
    favorites.addMerchantFavorite(props.merchant);
  }
}

function openWebsite() {
  if (props.merchant.website) {
    window.open(props.merchant.website, "_blank");
  }
}

function callPhone() {
  if (props.merchant.phone) {
    window.location.href = `tel:${props.merchant.phone}`;
  }
}

function sendEmail() {
  if (props.merchant.email) {
    window.location.href = `mailto:${props.merchant.email}`;
  }
}

function openSocialMedia(platformStr: string) {
  const socialEntry = props.merchant.social_media?.find((s) => {
    const p = typeof s.platform === "string" ? s.platform : s.platform.other;
    return p === platformStr;
  });
  if (socialEntry) {
    if (socialEntry.url) {
      window.open(socialEntry.url, "_blank");
    }
    // TODO: Implement platform-specific URL schemes for platforms without URL
  }
}

function getSocialIcon(platform: string): string {
  const iconMap: Record<string, string> = {
    wechat: "mdi:wechat",
    weibo: "simple-icons:sinaweibo",
    tiktok: "simple-icons:tiktok",
    facebook: "mdi:facebook",
    instagram: "mdi:instagram",
    twitter: "mdi:twitter",
    linkedin: "mdi:linkedin",
    youtube: "mdi:youtube",
    reddit: "mdi:reddit",
    discord: "mdi:discord",
    whatsapp: "mdi:whatsapp",
    telegram: "mdi:telegram",
    rednote: "mdi:note",
    bluesky: "mdi:cloud",
    bilibili: "mdi:video",
  };
  return iconMap[platform] || "mdi:link";
}
</script>

<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-start justify-between">
      <div class="flex-1">
        <h2 class="text-2xl font-bold text-gray-900 dark:text-white">
          {{ merchant.name }}
        </h2>
        <p class="text-gray-600 dark:text-gray-400 mt-1">
          {{ merchant.description }}
        </p>
        <div class="flex flex-wrap gap-2 mt-3">
          <Badge variant="secondary">
            <Icon icon="mdi:store" class="w-3 h-3 mr-1" />
            {{ merchantTypeLabel }}
          </Badge>
          <Badge v-if="merchant.style" variant="outline">
            <Icon icon="mdi:format-paint" class="w-3 h-3 mr-1" />
            {{ merchant.style }}
          </Badge>
        </div>
      </div>
      <div class="flex gap-2">
        <Button variant="ghost" size="sm" @click="toggleFavorite">
          <Icon
            :icon="isFavorited ? 'mdi:heart' : 'mdi:heart-outline'"
            :class="['w-5 h-5', isFavorited ? 'text-red-500' : '']"
          />
        </Button>
        <Button variant="ghost" size="sm" @click="emit('close')">
          <Icon icon="mdi:close" class="w-5 h-5" />
        </Button>
      </div>
    </div>

    <!-- Tags -->
    <div
      v-if="merchant.tags && merchant.tags.length > 0"
      class="flex flex-wrap gap-2"
    >
      <Badge
        v-for="tag in merchant.tags"
        :key="tag"
        variant="outline"
        class="text-xs"
      >
        <Icon icon="mdi:tag" class="w-3 h-3 mr-1" />
        {{ tag }}
      </Badge>
    </div>

    <Separator />

    <!-- Tabs -->
    <div class="flex gap-2 border-b border-gray-200 dark:border-gray-700">
      <button
        @click="activeTab = 'details'"
        :class="[
          'px-4 py-2 font-medium text-sm border-b-2 transition-colors',
          activeTab === 'details'
            ? 'border-blue-500 text-blue-600 dark:text-blue-400'
            : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200',
        ]"
      >
        <Icon icon="mdi:information-outline" class="inline w-4 h-4 mr-1" />
        Details
      </button>
      <button
        @click="activeTab = 'contact'"
        :class="[
          'px-4 py-2 font-medium text-sm border-b-2 transition-colors',
          activeTab === 'contact'
            ? 'border-blue-500 text-blue-600 dark:text-blue-400'
            : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200',
        ]"
      >
        <Icon icon="mdi:contact-mail" class="inline w-4 h-4 mr-1" />
        Contact
      </button>
      <button
        @click="activeTab = 'hours'"
        :class="[
          'px-4 py-2 font-medium text-sm border-b-2 transition-colors',
          activeTab === 'hours'
            ? 'border-blue-500 text-blue-600 dark:text-blue-400'
            : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200',
        ]"
      >
        <Icon icon="mdi:clock-outline" class="inline w-4 h-4 mr-1" />
        Hours
      </button>
    </div>

    <!-- Tab Content -->
    <div class="py-4">
      <!-- Details Tab -->
      <div v-if="activeTab === 'details'" class="space-y-4">
        <div v-if="merchant.chain" class="flex items-center gap-2">
          <Icon icon="mdi:store-outline" class="w-5 h-5 text-gray-400" />
          <div>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              Chain
            </p>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ merchant.chain }}
            </p>
          </div>
        </div>

        <Separator />

        <div class="flex items-center gap-2">
          <Icon icon="mdi:map-marker" class="w-5 h-5 text-gray-400" />
          <div>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              Location
            </p>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              Coordinates: ({{ merchant.location[0].toFixed(1) }},
              {{ merchant.location[1].toFixed(1) }})
            </p>
          </div>
        </div>

        <Separator />

        <div v-if="merchant.beacon_code" class="flex items-center gap-2">
          <Icon icon="mdi:bluetooth" class="w-5 h-5 text-gray-400" />
          <div>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              Beacon Code
            </p>
            <p class="text-sm font-mono text-gray-600 dark:text-gray-400">
              {{ merchant.beacon_code }}
            </p>
          </div>
        </div>
      </div>

      <!-- Contact Tab -->
      <div v-if="activeTab === 'contact'" class="space-y-4">
        <div v-if="merchant.phone" class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Icon icon="mdi:phone" class="w-5 h-5 text-gray-400" />
            <div>
              <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Phone
              </p>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                {{ merchant.phone }}
              </p>
            </div>
          </div>
          <Button size="sm" variant="outline" @click="callPhone">
            <Icon icon="mdi:phone" class="w-4 h-4 mr-1" />
            Call
          </Button>
        </div>

        <Separator v-if="merchant.phone" />

        <div v-if="merchant.email" class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Icon icon="mdi:email" class="w-5 h-5 text-gray-400" />
            <div>
              <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Email
              </p>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                {{ merchant.email }}
              </p>
            </div>
          </div>
          <Button size="sm" variant="outline" @click="sendEmail">
            <Icon icon="mdi:email" class="w-4 h-4 mr-1" />
            Email
          </Button>
        </div>

        <Separator v-if="merchant.email" />

        <div v-if="merchant.website" class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Icon icon="mdi:web" class="w-5 h-5 text-gray-400" />
            <div>
              <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Website
              </p>
              <p
                class="text-sm text-gray-600 dark:text-gray-400 truncate max-w-xs"
              >
                {{ merchant.website }}
              </p>
            </div>
          </div>
          <Button size="sm" variant="outline" @click="openWebsite">
            <Icon icon="mdi:open-in-new" class="w-4 h-4 mr-1" />
            Visit
          </Button>
        </div>

        <Separator v-if="merchant.website" />

        <!-- Social Media -->
        <div
          v-if="merchant.social_media && merchant.social_media.length > 0"
          class="space-y-3"
        >
          <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
            Social Media
          </p>
          <div class="flex gap-2 flex-wrap">
            <Button
              v-for="(social, idx) in merchant.social_media.map((s) => ({
                ...s,
                platformStr:
                  typeof s.platform === 'string'
                    ? s.platform
                    : s.platform.other,
              }))"
              :key="idx"
              size="sm"
              variant="outline"
              @click="openSocialMedia(social.platformStr)"
            >
              <Icon
                :icon="getSocialIcon(social.platformStr)"
                class="w-4 h-4 mr-1"
              />
              {{ social.platformStr }}
            </Button>
          </div>
        </div>

        <div
          v-if="
            !merchant.phone &&
            !merchant.email &&
            !merchant.website &&
            !merchant.social_media
          "
          class="text-center py-8"
        >
          <Icon
            icon="mdi:contact-mail-outline"
            class="w-12 h-12 mx-auto mb-2 text-gray-300 dark:text-gray-600"
          />
          <p class="text-sm text-gray-500 dark:text-gray-400">
            No contact information available
          </p>
        </div>
      </div>

      <!-- Hours Tab -->
      <div v-if="activeTab === 'hours'" class="space-y-4">
        <!-- TODO: Implement business hours when schema is extended -->
        <div class="text-center py-8">
          <Icon
            icon="mdi:clock-outline"
            class="w-12 h-12 mx-auto mb-2 text-gray-300 dark:text-gray-600"
          />
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Business hours not available
          </p>
          <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
            Check contact information for details
          </p>
        </div>
      </div>
    </div>

    <Separator />

    <!-- Navigation Button -->
    <Button class="w-full" @click="emit('navigate')">
      <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
      Navigate to {{ merchant.name }}
    </Button>
  </div>
</template>
