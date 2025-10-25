<script setup lang="ts">
import { Entity } from "@/schema";
import { toRefs } from "vue";

const props = defineProps<{
  entity: Entity;
}>();

const { entity } = toRefs(props);
</script>

<template></template>

<style scoped></style>
