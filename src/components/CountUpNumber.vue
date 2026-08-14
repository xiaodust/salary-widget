<script setup lang="ts">
import { onUnmounted, ref, watch } from "vue";

const props = defineProps<{
  target: number;
  rate: number;
  active: boolean;
  animations: boolean;
}>();

const display = ref(props.animations ? 0 : props.target);
let raf = 0;
let last = performance.now();

function stop() {
  if (raf !== 0) {
    cancelAnimationFrame(raf);
    raf = 0;
  }
}

function shouldRun() {
  return (
    props.animations &&
    (props.active || Math.abs(props.target - display.value) > 0.001)
  );
}

function start() {
  if (raf !== 0) return;
  last = performance.now();
  raf = requestAnimationFrame(loop);
}

function sync() {
  if (!props.animations) {
    display.value = props.target;
    stop();
    return;
  }

  if (shouldRun()) {
    start();
  } else {
    display.value = props.target;
    stop();
  }
}

function loop(now: number) {
  const dt = (now - last) / 1000;
  last = now;

  if (props.active && props.rate > 0) {
    display.value += props.rate * dt;
  }

  const diff = props.target - display.value;
  if (Math.abs(diff) > 0.001) {
    display.value += diff * Math.min(1, dt * 8);
  } else {
    display.value = props.target;
  }

  if (shouldRun()) {
    raf = requestAnimationFrame(loop);
  } else {
    raf = 0;
  }
}

watch(
  () => [props.target, props.active, props.animations],
  sync,
  { immediate: true },
);

onUnmounted(stop);
</script>

<template>
  <span class="num">{{ display.toFixed(2) }}</span>
</template>
