import { reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

export type WorkStatus =
  | "before_work"
  | "working"
  | "lunch"
  | "after_work"
  | "rest_day";

export interface Snapshot {
  now: string;
  status: WorkStatus;
  earned_today: number;
  work_seconds: number;
  remaining_seconds: number | null;
  rate_per_second: number;
  progress: number;
  day_total_seconds: number;
  lunch_seconds: number;
}

export interface Config {
  salary_type: "monthly" | "hourly" | "daily";
  monthly_salary: number;
  daily_salary: number;
  hourly_rate: number;
  workdays_per_month: number;
  daily_hours: number;
  work_start: string;
  work_end: string;
  lunch_start: string;
  lunch_end: string;
  lunch_enabled: boolean;
  workdays: number[];
  display_mode: "topmost" | "desktop" | "normal";
  compact: boolean;
  animations: boolean;
  locked: boolean;
  autostart: boolean;
  pos_x: number | null;
  pos_y: number | null;
  scale: number;
}

export const salary = reactive({
  cfg: null as Config | null,
  snap: null as Snapshot | null,
  settingsOpen: false,
  notice: "" as string,
  noticeKind: "info" as "info" | "error" | "success",
});

let unlisten: UnlistenFn | undefined;
let noticeTimer: number | undefined;

export function showNotice(
  message: string,
  kind: "info" | "error" | "success" = "info",
  duration = 4000,
): void {
  salary.notice = message;
  salary.noticeKind = kind;
  if (noticeTimer !== undefined) {
    window.clearTimeout(noticeTimer);
  }
  noticeTimer = window.setTimeout(() => {
    salary.notice = "";
    noticeTimer = undefined;
  }, duration);
}

export async function initSalary(): Promise<void> {
  salary.cfg = await invoke<Config>("get_config");
  if (!unlisten) {
    await listen("desktop-unavailable", () => {
      showNotice("当前系统不支持桌面层模式，已退回普通模式", "error");
    });
    await listen("open-settings", () => {
      salary.settingsOpen = true;
    });
    await listen<Config>("config-updated", (e) => {
      salary.cfg = e.payload;
    });
    unlisten = await listen<Snapshot>("salary:tick", (e) => {
      salary.snap = e.payload;
    });
  }
}

export async function saveConfig(cfg: Config): Promise<void> {
  await invoke("save_config", { config: cfg });
  salary.cfg = cfg;
}

export async function setMode(mode: string): Promise<void> {
  await invoke("set_display_mode", { mode });
  if (salary.cfg) {
    salary.cfg.display_mode = mode as Config["display_mode"];
  }
}

export async function savePosition(x: number, y: number): Promise<void> {
  await invoke("save_position", { x, y });
}

export async function createShortcut(): Promise<void> {
  await invoke("create_shortcut");
}

export async function toggleAutostart(enabled: boolean): Promise<void> {
  await invoke("set_autostart", { enabled });
}

export async function setLocked(enabled: boolean): Promise<void> {
  await invoke("set_locked", { enabled });
}

export async function quitApp(): Promise<void> {
  await invoke("quit_app");
}
