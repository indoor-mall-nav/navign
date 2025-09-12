<script setup lang="ts">
import type { Merchant as TypeMerchant, Area as TypeArea } from "@/schema";
import { ref } from "vue";
import { mockAreas, mockMerchants, polygonMock } from "@/mocks.ts";
import Polygon from "@/components/map/Polygon.tsx";
import Merchant from "@/components/cards/Merchant.vue";
import Area from "@/components/cards/Area.vue";

const merchants = ref<TypeMerchant[]>([mockMerchants[0]]);

const areas = ref<TypeArea[]>([mockAreas[0]]);

const polygons = ref<any[]>(polygonMock); // Replace 'any' with the appropriate type if available
</script>

<template>
  <div class="mx-4 mt-16">
    <Area v-for="area in areas" :key="area._id.$oid" :area="area" />
    <Merchant
      v-for="merchant in merchants"
      :key="merchant._id.$oid"
      :merchant="merchant"
    />
    <!--    <Polygon v-for="(polygon, index) in polygons" :key="index" :points="polygon.points" :fill="polygon.fill" :stroke="polygon.stroke" :stroke-width="polygon.strokeWidth" />-->
  </div>
</template>

<style scoped></style>
