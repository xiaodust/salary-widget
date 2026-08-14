<script setup lang="ts">
import { computed } from "vue";
import StatusBadge from "./StatusBadge.vue";
import CountUpNumber from "./CountUpNumber.vue";
import { salary } from "../stores/salary";
import { fmtDuration, modeText, statusText } from "../lib/format";

defineProps<{ dragging: boolean }>();

const snap = computed(() => salary.snap);
const status = computed(() => snap.value?.status ?? "before_work");
const isWorking = computed(() => status.value === "working");
const pct = computed(() =>
  Math.min(100, Math.round((snap.value?.progress ?? 0) * 100)),
);
const remainLabel = computed(() =>
  snap.value?.remaining_seconds != null
    ? fmtDuration(snap.value.remaining_seconds)
    : "--:--:--",
);
const rateLabel = computed(() => {
  const rps = snap.value?.rate_per_second ?? 0;
  return `+¥${(rps * 60).toFixed(2)}/分钟`;
});
const mode = computed(() => modeText[salary.cfg?.display_mode ?? "topmost"] ?? "置顶");
const animOn = computed(() => salary.cfg?.animations ?? true);
const compact = computed(() => salary.cfg?.compact ?? false);
</script>

<template>
  <div
    class="card"
    :class="[
      `status-${status}`,
      { dragging, compact, 'anim-on': animOn && status === 'working' },
    ]"
  >
    <div class="header">
      <span class="time">{{ snap?.now ?? "--:--:--" }}</span>
      <div class="header-right">
        <span
          v-if="salary.cfg?.locked"
          class="lock-badge"
          title="已锁定：鼠标点击穿透到下方应用。解锁方法：点系统托盘图标 → 解锁（恢复交互）"
        >🔒</span>
        <StatusBadge :status="status" />
      </div>
    </div>
    <div class="amount" :class="{ idle: !isWorking }">
      <span class="currency">¥</span>
      <CountUpNumber
        :target="snap?.earned_today ?? 0"
        :rate="snap?.rate_per_second ?? 0"
        :active="isWorking"
        :animations="animOn"
      />
    </div>
    <div v-if="!compact" class="sub">
      {{ rateLabel }} · 已工作 {{ fmtDuration(snap?.work_seconds ?? 0) }}
    </div>
    <div v-if="!compact" class="progress-wrap">
      <div class="progress" :style="{ width: pct + '%' }"></div>
    </div>
    <div v-if="!compact" class="footer">
      <span v-if="status === 'working'">距离下班 {{ remainLabel }}</span>
      <span v-else>{{ statusText[status] }}</span>
      <span class="mode-tag">{{ mode }}</span>
    </div>
  </div>
</template>
