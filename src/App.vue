<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { getCurrentWindow, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import SalaryCard from "./components/SalaryCard.vue";
import SettingsView from "./components/SettingsView.vue";
import ContextMenu from "./components/ContextMenu.vue";
import { initSalary, salary, savePosition, showNotice } from "./stores/salary";

const appWindow = getCurrentWindow();
const dragging = ref(false);
const menu = ref({ x: 0, y: 0, visible: false });
// 手动指针拖动（仅兜底）：原生拖动不可用或静默失败时自动启用
let manualDrag: {
  active: boolean;
  startX: number;
  startY: number;
  winX: number;
  winY: number;
  scale: number;
} | null = null;
let nativeDragTimer: number | null = null;
let saveTimer: number | null = null;
let unlistenMove: (() => void) | undefined;

onMounted(async () => {
  await initSalary();
  // 监听窗口移动：判断原生拖动是否真的生效，并在拖动结束后保存位置
  unlistenMove = await appWindow.onMoved(() => {
    if (nativeDragTimer !== null) {
      clearTimeout(nativeDragTimer);
      nativeDragTimer = null;
    }
    if (saveTimer !== null) {
      clearTimeout(saveTimer);
    }
    saveTimer = window.setTimeout(async () => {
      await persistPosition();
    }, 800);
  });
});

// 视图切换时同步调整窗口尺寸（逻辑像素，适配任意 DPI）
watch(
  () => [salary.settingsOpen, salary.cfg?.compact],
  async () => {
    const w = salary.settingsOpen ? 520 : 300;
    const h = salary.settingsOpen ? 640 : salary.cfg?.compact ? 92 : 190;
    await appWindow.setSize(new LogicalSize(w, h));
  },
  { immediate: true },
);

async function startDrag(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.closest("button, input, select, textarea, label, .no-drag")) return;
  if (e.button !== 0) return;
  if (salary.cfg?.locked) return;
  if (menu.value.visible) {
    // 菜单打开时点击应先收起菜单，而不是拖动窗口
    return;
  }
  dragging.value = true;
  try {
    // 原生拖动生效时窗口会立即移动（触发 onMoved 并清除定时器）。
    // 若 350ms 内窗口没有动，说明系统拖动静默失败，自动降级为手动拖动。
    nativeDragTimer = window.setTimeout(() => {
      nativeDragTimer = null;
      beginManualDrag(e);
    }, 350);
    await appWindow.startDragging();
    if (nativeDragTimer !== null) {
      clearTimeout(nativeDragTimer);
      nativeDragTimer = null;
    }
  } catch (err) {
    if (nativeDragTimer !== null) {
      clearTimeout(nativeDragTimer);
      nativeDragTimer = null;
    }
    // 原生拖动不可用（极少数环境），降级为手动拖动
    console.warn("native drag unavailable, fallback to manual", err);
    beginManualDrag(e);
    return;
  }
  if (!manualDrag?.active) {
    dragging.value = false;
  }
}

async function beginManualDrag(e: MouseEvent) {
  const pos = await appWindow.outerPosition();
  const scale = await appWindow.scaleFactor();
  manualDrag = {
    active: true,
    startX: e.clientX,
    startY: e.clientY,
    winX: pos.x,
    winY: pos.y,
    scale,
  };
  window.addEventListener("pointermove", onManualMove);
  window.addEventListener("pointerup", onManualUp);
  window.addEventListener("pointercancel", onManualUp);
}

async function onManualMove(e: PointerEvent) {
  if (!manualDrag?.active) return;
  // 鼠标左键已松开则结束手动拖动（防止普通悬停误移窗口）
  if (!(e.buttons & 1)) {
    await onManualUp();
    return;
  }
  const dx = (e.clientX - manualDrag.startX) * manualDrag.scale;
  const dy = (e.clientY - manualDrag.startY) * manualDrag.scale;
  await appWindow.setPosition(
    new PhysicalPosition(manualDrag.winX + Math.round(dx), manualDrag.winY + Math.round(dy)),
  );
}

async function onManualUp() {
  if (!manualDrag?.active) return;
  manualDrag = null;
  dragging.value = false;
  window.removeEventListener("pointermove", onManualMove);
  window.removeEventListener("pointerup", onManualUp);
  window.removeEventListener("pointercancel", onManualUp);
  await persistPosition();
}

async function persistPosition() {
  try {
    const pos = await appWindow.outerPosition();
    await savePosition(pos.x, pos.y);
  } catch (err) {
    console.warn("save position failed", err);
    showNotice("窗口位置保存失败", "error");
  }
}

function openMenu(e: MouseEvent) {
  menu.value = { x: e.clientX, y: e.clientY, visible: true };
}

function onContextMenu(e: MouseEvent) {
  e.preventDefault();
  openMenu(e);
}

function onRootClick() {
  if (menu.value.visible) {
    menu.value.visible = false;
  }
}
</script>

<template>
  <div class="root" @mousedown="startDrag" @contextmenu="onContextMenu" @click="onRootClick">
    <SalaryCard v-if="!salary.settingsOpen" :dragging="dragging" />
    <SettingsView v-else />
    <div v-if="salary.notice" class="notice" :class="salary.noticeKind">{{ salary.notice }}</div>
    <ContextMenu
      :x="menu.x"
      :y="menu.y"
      :visible="menu.visible"
      @close="menu.visible = false"
    />
  </div>
</template>
