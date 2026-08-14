<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  salary,
  setMode,
  toggleAutostart,
  setLocked,
  createShortcut,
  quitApp,
  saveConfig,
  showNotice,
} from "../stores/salary";

const props = defineProps<{ x: number; y: number; visible: boolean }>();
const emit = defineEmits<{ (e: "close"): void }>();
const menuRef = ref<HTMLElement | null>(null);
const pos = ref({ x: props.x, y: props.y });

watch(
  () => [props.visible, props.x, props.y],
  async () => {
    if (!props.visible) return;
    await nextTick();
    const el = menuRef.value;
    if (!el) return;
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    const maxX = Math.max(4, window.innerWidth - w - 4);
    const maxY = Math.max(4, window.innerHeight - h - 4);
    pos.value = {
      x: Math.min(Math.max(4, props.x), maxX),
      y: Math.min(Math.max(4, props.y), maxY),
    };
  },
  { immediate: true },
);

function close() {
  emit("close");
}

function errText(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function onGlobalClick() {
  if (props.visible) close();
}

onMounted(() => {
  window.addEventListener("click", onGlobalClick);
  window.addEventListener("blur", onGlobalClick);
});
onBeforeUnmount(() => {
  window.removeEventListener("click", onGlobalClick);
  window.removeEventListener("blur", onGlobalClick);
});

async function onMode(m: string) {
  try {
    await setMode(m);
    close();
  } catch (err) {
    showNotice(`切换显示模式失败：${errText(err)}`, "error");
    close();
  }
}

async function onAutostart() {
  if (!salary.cfg) return close();
  const v = !salary.cfg.autostart;
  try {
    await toggleAutostart(v);
    if (salary.cfg) salary.cfg.autostart = v;
    showNotice(v ? "已开启开机自启" : "已关闭开机自启", "success", 1800);
    close();
  } catch (err) {
    showNotice(`设置开机自启失败：${errText(err)}`, "error");
    close();
  }
}

async function onShortcut() {
  try {
    await createShortcut();
    showNotice("已创建桌面快捷方式", "success", 1800);
    close();
  } catch (err) {
    showNotice(`创建快捷方式失败：${errText(err)}`, "error");
    close();
  }
}

async function onLock() {
  if (!salary.cfg) return close();
  const v = !salary.cfg.locked;
  try {
    await setLocked(v);
    if (salary.cfg) salary.cfg.locked = v;
    showNotice(
      v
        ? "已锁定：鼠标点击穿透。解锁请点系统托盘图标 → 解锁"
        : "已解锁，恢复正常交互",
      "success",
      4000,
    );
    close();
  } catch (err) {
    showNotice(`设置锁定失败：${errText(err)}`, "error");
    close();
  }
}

function onSettings() {
  salary.settingsOpen = true;
  close();
}

async function onAnimations() {
  if (!salary.cfg) return close();
  const next = { ...salary.cfg, animations: !salary.cfg.animations };
  try {
    await saveConfig(next);
    showNotice(next.animations ? "已开启动画效果" : "已关闭动画效果", "success", 1600);
    close();
  } catch (err) {
    showNotice(`保存动画设置失败：${errText(err)}`, "error");
    close();
  }
}

function onQuit() {
  quitApp();
}
</script>

<template>
  <div
    v-if="visible"
    ref="menuRef"
    class="ctx-menu"
    :style="{ left: pos.x + 'px', top: pos.y + 'px' }"
    @click.stop
  >
    <div class="ctx-title">显示模式</div>
    <button
      class="ctx-item"
      :class="{ on: salary.cfg?.display_mode === 'topmost' }"
      @click="onMode('topmost')"
    >
      📌 置顶模式
    </button>
    <button
      class="ctx-item"
      :class="{ on: salary.cfg?.display_mode === 'desktop' }"
      @click="onMode('desktop')"
    >
      🖥 桌面模式
    </button>
    <button
      class="ctx-item"
      :class="{ on: salary.cfg?.display_mode === 'normal' }"
      @click="onMode('normal')"
    >
      📄 普通模式
    </button>
    <div class="ctx-sep"></div>
    <button class="ctx-item" :class="{ on: salary.cfg?.locked }" @click="onLock">
      {{ salary.cfg?.locked ? "🔓 解锁" : "🔒 锁定（点击穿透）" }}
    </button>
    <button class="ctx-item" @click="onSettings">⚙ 设置</button>
    <button class="ctx-item" @click="onShortcut">🖱 创建桌面快捷方式</button>
    <button class="ctx-item" :class="{ on: salary.cfg?.autostart }" @click="onAutostart">
      🔄 开机自启
    </button>
    <button class="ctx-item" :class="{ on: salary.cfg?.animations }" @click="onAnimations">
      ✨ 动画效果
    </button>
    <div class="ctx-sep"></div>
    <button class="ctx-item danger" @click="onQuit">✕ 退出</button>
  </div>
</template>
