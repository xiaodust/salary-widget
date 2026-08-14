<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import {
  salary,
  saveConfig,
  toggleAutostart,
  createShortcut,
  quitApp,
  showNotice,
  type Config,
} from "../stores/salary";

const cfg = ref<Config | null>(null);
const saved = ref(false);
const shortcutMsg = ref("");

onMounted(() => {
  cfg.value = salary.cfg ? JSON.parse(JSON.stringify(salary.cfg)) : null;
  syncDailyHours();
});

function timeToMinutes(value: string): number | null {
  const match = /^([01]\d|2[0-3]):([0-5]\d)$/.exec(value);
  if (!match) return null;
  return Number(match[1]) * 60 + Number(match[2]);
}

function calculateDailyHours(config: Config): number {
  const start = timeToMinutes(config.work_start);
  const end = timeToMinutes(config.work_end);
  if (start === null || end === null || end <= start) return 0;

  let minutes = end - start;
  if (config.lunch_enabled) {
    const lunchStart = timeToMinutes(config.lunch_start);
    const lunchEnd = timeToMinutes(config.lunch_end);
    if (
      lunchStart !== null &&
      lunchEnd !== null &&
      lunchStart >= start &&
      lunchEnd <= end &&
      lunchEnd > lunchStart
    ) {
      minutes -= lunchEnd - lunchStart;
    }
  }

  return Math.round((minutes / 60) * 10) / 10;
}

function syncDailyHours() {
  if (!cfg.value || cfg.value.salary_type === "hourly") return;
  cfg.value.daily_hours = calculateDailyHours(cfg.value);
}

watch(
  () =>
    cfg.value
      ? [
          cfg.value.salary_type,
          cfg.value.work_start,
          cfg.value.work_end,
          cfg.value.lunch_enabled,
          cfg.value.lunch_start,
          cfg.value.lunch_end,
        ]
      : [],
  syncDailyHours,
);

const weekdays = [
  { v: 1, label: "一" },
  { v: 2, label: "二" },
  { v: 3, label: "三" },
  { v: 4, label: "四" },
  { v: 5, label: "五" },
  { v: 6, label: "六" },
  { v: 7, label: "日" },
];

function flashSaved() {
  saved.value = true;
  setTimeout(() => (saved.value = false), 1600);
}

function errText(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function validateLocal(config: Config): string | null {
  if (!config.workdays.length) return "请至少选择一个工作日";
  if (!Number.isFinite(config.monthly_salary) || config.monthly_salary < 0) {
    return "月薪必须是不小于 0 的数字";
  }
  if (!Number.isFinite(config.daily_salary) || config.daily_salary < 0) {
    return "日薪必须是不小于 0 的数字";
  }
  if (!Number.isFinite(config.hourly_rate) || config.hourly_rate < 0) {
    return "时薪必须是不小于 0 的数字";
  }
  if (
    !Number.isFinite(config.workdays_per_month) ||
    config.workdays_per_month <= 0 ||
    !Number.isFinite(config.daily_hours) ||
    config.daily_hours <= 0
  ) {
    return "月工作天数和每日小时数必须大于 0";
  }

  const timePattern = /^([01]\d|2[0-3]):[0-5]\d$/;
  if (!timePattern.test(config.work_start) || !timePattern.test(config.work_end)) {
    return "上下班时间格式应为 HH:mm";
  }
  if (config.work_start >= config.work_end) {
    return "下班时间必须晚于上班时间，暂不支持跨天排班";
  }

  if (config.lunch_enabled) {
    if (!timePattern.test(config.lunch_start) || !timePattern.test(config.lunch_end)) {
      return "午休时间格式应为 HH:mm";
    }
    if (config.lunch_start >= config.lunch_end) {
      return "午休结束时间必须晚于开始时间";
    }
    if (config.lunch_start < config.work_start || config.lunch_end > config.work_end) {
      return "午休时间必须在上、下班时间范围内";
    }
  }

  return null;
}

async function save() {
  if (!cfg.value) return;
  const error = validateLocal(cfg.value);
  if (error) {
    showNotice(error, "error");
    return;
  }
  try {
    await saveConfig(cfg.value);
    flashSaved();
    showNotice("设置已保存", "success", 1600);
  } catch (err) {
    showNotice(`保存失败：${errText(err)}`, "error");
  }
}

async function onAutostartChange() {
  if (!cfg.value) return;
  const target = cfg.value.autostart;
  try {
    await toggleAutostart(target);
    showNotice(target ? "已开启开机自启" : "已关闭开机自启", "success", 1800);
  } catch (err) {
    cfg.value.autostart = salary.cfg?.autostart ?? !target;
    showNotice(`设置开机自启失败：${errText(err)}`, "error");
  }
}

async function onShortcut() {
  try {
    await createShortcut();
    shortcutMsg.value = "已创建到桌面";
    setTimeout(() => (shortcutMsg.value = ""), 2000);
  } catch (err) {
    showNotice(`创建快捷方式失败：${errText(err)}`, "error");
  }
}

function back() {
  salary.settingsOpen = false;
}
</script>

<template>
  <div class="settings card">
    <div class="settings-header">
      <span class="settings-title">⚙ 设置</span>
      <button class="btn-ghost" @click="back">返回</button>
    </div>

    <div v-if="cfg" class="settings-body">
      <div class="field">
        <label>计薪方式</label>
        <select v-model="cfg.salary_type">
          <option value="monthly">月薪</option>
          <option value="daily">日薪</option>
          <option value="hourly">时薪</option>
        </select>
      </div>

      <template v-if="cfg.salary_type === 'monthly'">
        <div class="field">
          <label>月薪（元）</label>
          <input v-model.number="cfg.monthly_salary" type="number" min="0" step="100" />
        </div>
        <div class="field-row">
          <div class="field">
            <label>月工作天数</label>
            <input v-model.number="cfg.workdays_per_month" type="number" min="1" max="31" step="0.5" />
          </div>
          <div class="field">
            <label>每日小时</label>
            <input
              v-model.number="cfg.daily_hours"
              type="number"
              min="1"
              max="24"
              step="0.5"
              readonly
              title="由上下班时间和午休时间自动计算"
            />
          </div>
        </div>
      </template>
      <template v-else-if="cfg.salary_type === 'daily'">
        <div class="field">
          <label>日薪（元）</label>
          <input v-model.number="cfg.daily_salary" type="number" min="0" step="10" />
        </div>
        <div class="field">
          <label>每日小时</label>
          <input
            v-model.number="cfg.daily_hours"
            type="number"
            min="1"
            max="24"
            step="0.5"
            readonly
            title="由上下班时间和午休时间自动计算"
          />
        </div>
      </template>
      <div v-else class="field">
        <label>时薪（元）</label>
        <input v-model.number="cfg.hourly_rate" type="number" min="0" step="1" />
      </div>

      <div class="field-row">
        <div class="field">
          <label>上班时间</label>
          <input v-model="cfg.work_start" type="time" />
        </div>
        <div class="field">
          <label>下班时间</label>
          <input v-model="cfg.work_end" type="time" />
        </div>
      </div>

      <div class="field-check">
        <label><input v-model="cfg.lunch_enabled" type="checkbox" /> 午休</label>
      </div>
      <div v-if="cfg.lunch_enabled" class="field-row">
        <div class="field">
          <label>午休开始</label>
          <input v-model="cfg.lunch_start" type="time" />
        </div>
        <div class="field">
          <label>午休结束</label>
          <input v-model="cfg.lunch_end" type="time" />
        </div>
      </div>

      <div class="field">
        <label>工作日</label>
        <div class="weekday-row">
          <label
            v-for="d in weekdays"
            :key="d.v"
            class="weekday"
            :class="{ on: cfg.workdays.includes(d.v) }"
          >
            <input
              v-model="cfg.workdays"
              type="checkbox"
              :value="d.v"
              class="hidden-input"
            />
            {{ d.label }}
          </label>
        </div>
      </div>

      <div class="field">
        <label>显示模式</label>
        <div class="mode-row">
          <label v-for="m in [
            { v: 'topmost', t: '置顶' },
            { v: 'desktop', t: '桌面' },
            { v: 'normal', t: '普通' },
          ]" :key="m.v" class="mode-pill" :class="{ on: cfg.display_mode === m.v }">
            <input v-model="cfg.display_mode" type="radio" :value="m.v" class="hidden-input" />
            {{ m.t }}
          </label>
        </div>
      </div>

      <div class="field-check">
        <label><input v-model="cfg.compact" type="checkbox" /> 紧凑模式</label>
      </div>
      <div class="field-check">
        <label><input v-model="cfg.animations" type="checkbox" /> 动画效果</label>
      </div>
      <div class="field-check">
        <label>
          <input v-model="cfg.autostart" type="checkbox" @change="onAutostartChange" />
          开机自启
        </label>
      </div>

      <div class="actions">
        <button class="btn primary" @click="save">{{ saved ? "✓ 已保存" : "保存设置" }}</button>
        <button class="btn" @click="onShortcut">创建桌面快捷方式</button>
        <button class="btn danger" @click="quitApp">退出</button>
      </div>
      <div v-if="shortcutMsg" class="toast">{{ shortcutMsg }}</div>
    </div>
  </div>
</template>
