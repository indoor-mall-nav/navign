<script setup lang="ts">
import type { Merchant as TypeMerchant } from "@/schema";
import { ref } from 'vue';
import { mockMerchants } from "@/mocks.ts";
import Merchant from "@/components/cards/Merchant.vue";

const merchants = ref<TypeMerchant[]>(mockMerchants)
</script>

<template>
  <div class="mx-4 mt-16">
    <Merchant v-for="merchant in merchants" :key="merchant._id.$oid" :merchant="merchant" />
  </div>
</template>

<style scoped></style>
