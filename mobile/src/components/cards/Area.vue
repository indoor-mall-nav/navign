<script setup lang="ts">
import { Area } from '@/schema'
import { toRefs } from 'vue'

const props = defineProps<{
  area: Area
}>()

const { area } = toRefs(props)
</script>

<template>
  <div class="m-2">
    <p class="mx-4 text-xl flex items-center text-center">
      {{ area.name }}
    </p>
    <p class="mx-4 text-sm">
      {{ area.description }}
    </p>
  </div>
</template>

<style scoped></style>
