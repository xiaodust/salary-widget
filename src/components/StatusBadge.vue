<script setup lang="ts">
import { computed } from "vue";
import { statusText } from "../lib/format";

const props = defineProps<{ status: string }>();
const label = computed(() => statusText[props.status] ?? props.status);
</script>

<template>
  <span class="badge" :class="`st-${status}`">
    <span class="dot"></span>{{ label }}
  </span>
</template>
