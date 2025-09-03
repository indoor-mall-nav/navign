<script setup lang="ts">
import { Area } from "@/schema";
import { toRefs } from "vue";
import Polygon from '@/components/map/Polygon.tsx';

const props = defineProps<{
  area: Area;
}>();

const { area } = toRefs(props);

</script>

<template>
  <div class="m-2">
    <p class="mx-4 text-xl flex items-center text-center">
      {{ area.name }}
    </p>
    <p class="mx-4 text-sm">
      {{ area.description }}
    </p>
    <Polygon :points="area.polygon" fill="rgba(255, 0, 0, 0.3)" stroke="red" :stroke-width="2" />
  </div>
</template>

<style scoped></style>
