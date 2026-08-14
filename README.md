# Salary Widget

一个轻量级 Windows 桌面薪资实时显示挂件。设置月薪、日薪或时薪与上下班时间后，它会在桌面实时显示当前时间、工作状态和“此刻已经赚了多少钱”。

<img src="./app-icon.png" width="128" height="128" alt="Salary Widget" />

[![GitHub release](https://img.shields.io/github/v/release/xiaodust/salary-widget?label=Latest%20Release)](https://github.com/xiaodust/salary-widget/releases/latest)
[![License](https://img.shields.io/github/license/xiaodust/salary-widget)](./LICENSE)

## 功能一览

- 实时金额：按秒累计“今日已赚”，数字平滑滚动，带呼吸光晕与流光描边动画
- 三种计薪方式：月薪（默认按 21.75 天折算）、日薪、时薪
- 状态机：待上班 / 上班中 / 午休中 / 已下班 / 休息日
- 下班倒计时：显示距离下班的剩余时间
- 三种显示模式：置顶、桌面层、普通窗口
- 任意位置拖动，位置自动记忆
- 系统托盘菜单：显示/隐藏、模式切换、锁定、设置、开机自启、退出
- 开机自启与桌面快捷方式
- 紧凑模式与动画开关
- 单实例运行：重复启动会唤起已有窗口

## 下载与安装

### 方式一：安装包（推荐）

前往 [GitHub Releases](https://github.com/xiaodust/salary-widget/releases/latest) 下载：

```text
SalaryWidget_0.1.0_x64-setup.exe
```

安装步骤：

1. 双击 `SalaryWidget_0.1.0_x64-setup.exe`
2. 选择安装目录，建议使用默认目录
3. 安装完成后启动应用
4. 右键桌面挂件，选择“设置”，填写薪资和上下班时间
5. 点击“保存设置”即可生效

安装器为当前用户安装，不需要管理员权限。

### 方式二：便携版

前往 [GitHub Releases](https://github.com/xiaodust/salary-widget/releases/latest) 下载：

```text
SalaryWidget_0.1.0_x64-portable.zip
```

使用步骤：

1. 解压到任意目录，例如 `D:\SalaryWidget`
2. 确保目录内包含以下文件：

   ```text
   SalaryWidget.exe
   WebView2Loader.dll
   ```

3. 双击 `SalaryWidget.exe`
4. 右键挂件进入设置

便携版不会写入开始菜单或安装目录，直接删除解压目录即可卸载。

## 配置与数据

配置文件位于：

```text
%APPDATA%\com.codex.salarywidget\config.json
```

删除该文件可恢复默认配置。如果配置文件损坏，应用会自动备份为 `config.json.invalid` 并重置为默认值。

## 开发

### 环境要求

- Windows 10/11
- Node.js 18+
- Rust stable
- Microsoft C++ Build Tools（Windows Rust 工具链）
- WebView2 Runtime

### 安装依赖

```powershell
npm install
```

### 启动开发模式

```powershell
npm run tauri:dev
```

前端单独启动：

```powershell
npm run dev
```

### 前端构建

```powershell
npm run build
```

### 构建安装包

```powershell
npm run tauri:build
```

构建成功后，安装包默认位于：

```text
src-tauri/target/release/bundle/nsis/SalaryWidget_0.1.0_x64-setup.exe
```

## 目录结构

```text
salary-widget/
├── .github/
│   └── workflows/
│       └── release.yml
├── src/
│   ├── components/         # Vue 组件
│   ├── lib/                # 前端格式化与工具
│   ├── stores/             # 状态与 IPC
│   ├── App.vue
│   ├── main.ts
│   └── style.css
├── src-tauri/
│   ├── src/                # Rust 后端
│   ├── capabilities/
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   └── build-installer.ps1
├── package.json
├── vite.config.ts
└── README.md
```

## 发布流程

1. 更新 `CHANGELOG.md`
2. 创建 Git Tag：

   ```powershell
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. GitHub Actions 会自动构建 Windows 安装包和便携版压缩包
4. 构建完成后在 Release 页面检查并发布

## 已知限制

- 当前不支持跨天排班，例如夜班
- 暂不自动识别法定节假日
- 暂不支持加班倍率和加班收入累计
- 桌面层模式依赖 Windows `WorkerW` / `Progman`，部分精简系统可能自动降级为普通模式

## Roadmap

- 加班继续累计与加班倍率
- 法定节假日与调休配置
- 跨天排班
- 月累计、年累计收入
- 多语言与自定义货币符号

## License

[MIT](./LICENSE)

## Security

请参阅 [SECURITY.md](./SECURITY.md)。
