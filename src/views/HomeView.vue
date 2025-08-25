<script setup lang="ts">
import type { Merchant as TypeMerchant } from "@/schema";
import { ref } from 'vue';
import { mockMerchants, polygonMock } from "@/mocks.ts";
import Polygon from '@/components/map/Polygon.tsx';
import Merchant from "@/components/cards/Merchant.vue";

const merchants = ref<TypeMerchant[]>(mockMerchants)

const polygons = ref<any[]>(polygonMock); // Replace 'any' with the appropriate type if available
</script>

<template>
  <div class="mx-4 mt-16">
    <Merchant v-for="merchant in merchants" :key="merchant._id.$oid" :merchant="merchant" />
    <Polygon v-for="(polygon, index) in polygons" :key="index" :points="polygon.points" :fill="polygon.fill" :stroke="polygon.stroke" :stroke-width="polygon.strokeWidth" />
  </div>
</template>

<style scoped></style>
