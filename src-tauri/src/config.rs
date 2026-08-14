use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use chrono::{NaiveTime, Timelike};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub salary_type: String,
    pub monthly_salary: f64,
    pub daily_salary: f64,
    pub hourly_rate: f64,
    pub workdays_per_month: f64,
    pub daily_hours: f64,
    pub work_start: String,
    pub work_end: String,
    pub lunch_start: String,
    pub lunch_end: String,
    pub lunch_enabled: bool,
    pub workdays: Vec<u32>,
    pub display_mode: String,
    pub compact: bool,
    pub animations: bool,
    pub locked: bool,
    pub autostart: bool,
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
    pub scale: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            salary_type: "monthly".into(),
            monthly_salary: 15000.0,
            daily_salary: 500.0,
            hourly_rate: 0.0,
            workdays_per_month: 21.75,
            daily_hours: 8.0,
            work_start: "09:00".into(),
            work_end: "18:00".into(),
            lunch_start: "12:00".into(),
            lunch_end: "13:00".into(),
            lunch_enabled: true,
            workdays: vec![1, 2, 3, 4, 5],
            display_mode: "topmost".into(),
            compact: false,
            animations: true,
            locked: false,
            autostart: false,
            pos_x: None,
            pos_y: None,
            scale: 1.0,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.salary_type.as_str(), "monthly" | "daily" | "hourly") {
            return Err("不支持的计薪方式".into());
        }
        if !matches!(self.display_mode.as_str(), "topmost" | "desktop" | "normal") {
            return Err("不支持的显示模式".into());
        }
        if self.workdays.is_empty() {
            return Err("请至少选择一个工作日".into());
        }

        match self.salary_type.as_str() {
            "monthly" => {
                if !self.monthly_salary.is_finite() || self.monthly_salary < 0.0 {
                    return Err("月薪必须是不小于 0 的数字".into());
                }
                if self.workdays_per_month <= 0.0 || self.daily_hours <= 0.0 {
                    return Err("月计薪天数和每日小时数必须大于 0".into());
                }
            }
            "daily" => {
                if !self.daily_salary.is_finite() || self.daily_salary < 0.0 {
                    return Err("日薪必须是不小于 0 的数字".into());
                }
                if self.daily_hours <= 0.0 {
                    return Err("每日小时数必须大于 0".into());
                }
            }
            "hourly" => {
                if !self.hourly_rate.is_finite() || self.hourly_rate < 0.0 {
                    return Err("时薪必须是不小于 0 的数字".into());
                }
            }
            _ => {}
        }

        let parse = |s: &str, label: &str| {
            NaiveTime::parse_from_str(s, "%H:%M")
                .map(|t| t.num_seconds_from_midnight() / 60)
                .map_err(|_| format!("{label} 时间格式不正确，应为 HH:mm"))
        };
        let start = parse(&self.work_start, "上班时间")?;
        let end = parse(&self.work_end, "下班时间")?;
        if start >= end {
            return Err("下班时间必须晚于上班时间，暂不支持跨天排班".into());
        }

        if self.lunch_enabled {
            let lunch_s = parse(&self.lunch_start, "午休开始")?;
            let lunch_e = parse(&self.lunch_end, "午休结束")?;
            if lunch_s >= lunch_e {
                return Err("午休结束时间必须晚于开始时间".into());
            }
            if lunch_s < start || lunch_e > end {
                return Err("午休时间必须在上、下班时间范围内".into());
            }
        }

        if self.workdays.iter().any(|d| !(1..=7).contains(d)) {
            return Err("工作日数据不合法".into());
        }

        Ok(())
    }

    pub fn effective_hourly(&self) -> f64 {
        match self.salary_type.as_str() {
            "hourly" => self.hourly_rate,
            "daily" if self.daily_hours > 0.0 => self.daily_salary / self.daily_hours,
            _ if self.workdays_per_month > 0.0 && self.daily_hours > 0.0 => {
                self.monthly_salary / self.workdays_per_month / self.daily_hours
            }
            _ => 0.0,
        }
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("config.json"))
}

pub fn load(app: &AppHandle) -> Result<Config, String> {
    let path = config_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let cfg: Config = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            if let Err(reason) = cfg.validate() {
                let backup = path.with_extension("json.invalid");
                let _ = std::fs::copy(&path, &backup);
                let default_cfg = Config::default();
                save(app, &default_cfg)?;
                return Err(format!("配置不合法，已重置为默认值：{reason}"));
            }
            Ok(cfg)
        }
        Err(_) => {
            let cfg = Config::default();
            save(app, &cfg)?;
            Ok(cfg)
        }
    }
}

pub fn save(app: &AppHandle, cfg: &Config) -> Result<(), String> {
    cfg.validate()?;
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("config.json");
    let tmp = dir.join("config.json.tmp");
    let data = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Config;

    fn default_cfg() -> Config {
        Config::default()
    }

    #[test]
    fn default_config_is_valid() {
        assert!(default_cfg().validate().is_ok());
    }

    #[test]
    fn rejects_night_shift() {
        let mut cfg = default_cfg();
        cfg.work_start = "22:00".into();
        cfg.work_end = "06:00".into();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_lunch_outside_work_range() {
        let mut cfg = default_cfg();
        cfg.lunch_start = "17:00".into();
        cfg.lunch_end = "19:00".into();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_empty_workdays() {
        let mut cfg = default_cfg();
        cfg.workdays.clear();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn calculates_effective_hourly_rate() {
        let mut cfg = default_cfg();
        cfg.salary_type = "monthly".into();
        cfg.monthly_salary = 17400.0;
        cfg.workdays_per_month = 21.75;
        cfg.daily_hours = 8.0;
        assert!((cfg.effective_hourly() - 100.0).abs() < 0.0001);
    }
}
