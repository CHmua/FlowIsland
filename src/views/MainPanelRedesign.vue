<template>
    <div class="fi-shell" :class="shellThemeClass">
        <header class="fi-titlebar" data-tauri-drag-region>
            <div class="title-brand" data-tauri-drag-region>
                <span class="brand-mark">
                    <img :src="flowIslandLogo" alt="" />
                </span>
                <strong>FlowIsland</strong>
            </div>
            <div class="title-status">
                <i></i>
                <span>{{ isWidgetVisible ? '灵动岛运行中' : '灵动岛已关闭' }}</span>
            </div>
            <div class="window-actions">
                <button aria-label="最小化" @click="minimizeWindow">
                    <Minus :size="15" />
                </button>
                <button aria-label="关闭" @click="closeWindow">
                    <X :size="16" />
                </button>
            </div>
        </header>

        <div class="fi-body">
            <aside class="sidebar">
                <div class="brand-block">
                    <div class="logo-mark">
                        <img :src="flowIslandLogo" alt="FlowIsland" />
                    </div>
                    <div>
                        <h1>FlowIsland</h1>
                        <p>v{{ appVersion }}</p>
                    </div>
                </div>

                <nav class="nav-list">
                    <button v-for="item in navItems" :key="item.id" class="nav-item"
                        :class="{ 'is-active': activeSection === item.id }" @click="activeSection = item.id">
                        <component :is="item.icon" :size="18" />
                        <span>{{ item.label }}</span>
                        <em v-if="item.badge">{{ item.badge }}</em>
                    </button>
                </nav>

                <button class="update-link" :disabled="isChecking || isUpdating" @click="checkUpdate">
                    <CloudDownload :size="16" />
                    <span>{{ isUpdating ? `更新中 ${updateProgress}%` : (isChecking ? '检查中' : (hasNewVersion ? '发现新版本' : '检查更新')) }}</span>
                </button>
            </aside>

            <main class="main-content">
                <section v-show="activeSection === 'island'" class="page-layout single-page">
                    <div class="settings-pane full">
                        <PageHeader title="灵动岛" desc="集中设置外观、通知、窗口与启动行为">
                            <div class="head-toggle">
                                <span>启用灵动岛</span>
                                <label class="native-switch">
                                    <input type="checkbox" :checked="isWidgetVisible" @change="toggleWidget">
                                    <span></span>
                                </label>
                            </div>
                        </PageHeader>

                        <div class="category-grid">
                            <SettingGroup title="显示" eyebrow="Appearance">
                                <SettingRow title="主题模式" desc="控制灵动岛与控制台的明暗外观">
                                    <div class="segmented">
                                        <button :class="{ 'is-active': islandTheme === 'black' }"
                                            @click="islandTheme = 'black'">暗色</button>
                                        <button :class="{ 'is-active': islandTheme === 'white' }"
                                            @click="islandTheme = 'white'">浅色</button>
                                        <button :class="{ 'is-active': islandTheme === 'system' }"
                                            @click="islandTheme = 'system'">跟随系统</button>
                                    </div>
                                </SettingRow>
                                <SettingRow title="透明度" desc="调整灵动岛背景的可见程度">
                                    <RangeControl v-model="opacity" min="0" max="100" suffix="%" />
                                </SettingRow>
                            </SettingGroup>

                            <SettingGroup title="通知与提醒" eyebrow="Notifications">
                                <SettingRow title="系统通知" desc="接收 Windows 通知和系统状态提醒">
                                    <UiSwitch v-model="enableMsgNotify" @change="toggleMsgNotify" />
                                </SettingRow>
                                <SettingRow title="消息模式" desc="无消息时隐藏，收到消息后弹出">
                                    <UiSwitch v-model="msgModeEnabled" @change="toggleMsgMode" />
                                </SettingRow>
                            </SettingGroup>

                            <SettingGroup title="窗口与启动" eyebrow="Window">
                                <SettingRow title="锁定位置" desc="固定灵动岛，防止鼠标误拖动">
                                    <UiSwitch v-model="pinToTaskbar" @change="togglePinTaskbar" />
                                </SettingRow>
                                <SettingRow title="开机启动" desc="登录 Windows 后自动启动 FlowIsland">
                                    <UiSwitch v-model="autoStart" @change="toggleAutoStart" />
                                </SettingRow>
                            </SettingGroup>
                        </div>
                    </div>
                </section>

                <section v-show="activeSection === 'music'" class="page-layout single-page">
                    <div class="settings-pane full">
                        <PageHeader title="音乐与歌词" desc="管理媒体会话识别、歌词显示与同步偏移" />
                        <SettingGroup title="音乐与歌词" eyebrow="Media">
                            <SettingRow title="音乐识别" desc="检测主流音乐播放器的 Windows 媒体会话">
                                <UiSwitch v-model="enableMusicCtrl" />
                            </SettingRow>
                            <SettingRow title="显示歌词" desc="匹配到同步歌词时在歌曲名称下方显示当前句">
                                <UiSwitch v-model="showLyrics" />
                            </SettingRow>
                            <SettingRow title="同步偏移" :desc="`当前 ${lyricOffsetMs}ms，用于手动微调歌词时间`">
                                <RangeControl v-model="lyricOffsetMs" min="-3000" max="3000" step="100" suffix="ms" />
                            </SettingRow>
                        </SettingGroup>
                    </div>
                </section>

                <section v-show="activeSection === 'monitor'" class="page-layout single-page">
                    <div class="settings-pane full">
                        <PageHeader title="系统监控" desc="选择灵动岛中需要展示的硬件与网络状态">
                            <div class="head-toggle">
                                <span>显示监控岛</span>
                                <UiSwitch v-model="enableHardwareMonitor" />
                            </div>
                        </PageHeader>
                        <div class="metric-grid">
                            <MetricCard icon="cpu" title="CPU" :value="monitorCpuUsage" :detail="monitorCpuDetail" />
                            <MetricCard icon="memory" title="内存" :value="monitorMemoryUsage" :detail="monitorMemoryDetail" />
                            <MetricCard icon="gpu" title="显卡" :value="monitorGpuUsage" :detail="monitorGpuDetail" />
                            <MetricCard icon="drive" title="显存" :value="monitorVramUsage" :detail="monitorVramDetail" />
                            <MetricCard icon="network" title="网络" :value="downloadSpeed" :detail="`上传 ${uploadSpeed}`" />
                        </div>
                        <SettingGroup title="监控设置" eyebrow="Metrics">
                            <div class="check-grid">
                                <label class="check-item">
                                    <input v-model="showCpu" type="checkbox">
                                    <span>CPU</span>
                                </label>
                                <label class="check-item">
                                    <input v-model="showGpu" type="checkbox">
                                    <span>显卡与显存</span>
                                </label>
                                <label class="check-item">
                                    <input v-model="showMemory" type="checkbox">
                                    <span>内存</span>
                                </label>
                                <label class="check-item">
                                    <input v-model="showNetwork" type="checkbox">
                                    <span>网络</span>
                                </label>
                            </div>
                            <SettingRow title="状态页刷新频率" desc="控制此页面硬件数据的读取间隔">
                                <select v-model="monitorRefreshRate" class="select">
                                    <option value="2">2 秒</option>
                                    <option value="1">1 秒</option>
                                    <option value="5">5 秒</option>
                                </select>
                            </SettingRow>
                        </SettingGroup>
                    </div>
                </section>

                <section v-show="activeSection === 'about'" class="page-layout single-page">
                    <div class="settings-pane full">
                        <PageHeader title="关于与更新" desc="版本信息、更新入口和项目链接" />
                        <div class="about-card">
                            <div class="about-logo">
                                <img :src="flowIslandLogo" alt="FlowIsland" />
                            </div>
                            <div>
                                <h3>FlowIsland</h3>
                                <p>桌面灵动岛组件 · v{{ appVersion }}</p>
                            </div>
                            <div class="about-actions">
                                <button class="primary-btn" :disabled="isChecking || isUpdating" @click="checkUpdate">
                                    {{ isUpdating ? `更新中 ${updateProgress}%` : (isChecking ? '检查中' : '检查更新') }}
                                </button>
                                <button class="secondary-btn" @click="openProjectRepo">
                                    <Github :size="17" />
                                    GitHub
                                </button>
                            </div>
                        </div>
                    </div>
                </section>
            </main>
        </div>

        <Transition name="fade">
            <div v-if="dialog.visible" class="modal-overlay" @click.self="closeDialog">
                <div class="modal-card">
                    <h3>{{ dialog.title }}</h3>
                    <p>{{ dialog.message }}</p>
                    <div class="modal-actions">
                        <button v-if="dialog.isConfirm" class="secondary-btn" @click="closeDialog">取消</button>
                        <button class="primary-btn" @click="handleDialogConfirm">确定</button>
                    </div>
                </div>
            </div>
        </Transition>

        <Transition name="fade">
            <div v-if="updatePanelVisible" class="modal-overlay">
                <div class="modal-card update-modal">
                    <div class="update-modal-icon" :class="{ 'has-error': Boolean(updateError) }">
                        <CloudDownload v-if="!updateError" :size="24" />
                        <X v-else :size="23" />
                    </div>
                    <h3>{{ updateError ? '更新失败' : `正在更新到 ${updateVersion}` }}</h3>
                    <p>{{ updateError || updateStatus }}</p>
                    <div v-if="!updateError" class="update-progress" role="progressbar"
                        :aria-valuenow="updateProgress" aria-valuemin="0" aria-valuemax="100">
                        <i :style="{ width: `${updateProgress}%` }"></i>
                    </div>
                    <div v-if="!updateError" class="update-progress-meta">
                        <span>{{ updateDownloadedText }}</span>
                        <strong>{{ updateProgress }}%</strong>
                    </div>
                    <div v-else class="modal-actions">
                        <button class="secondary-btn" @click="closeUpdatePanel">关闭</button>
                        <button class="primary-btn" @click="retryUpdate">重试</button>
                    </div>
                </div>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
    Activity,
    CloudDownload,
    Cpu,
    Github,
    HardDrive,
    Info,
    Minus,
    Monitor,
    Music2,
    Sparkles,
    Wifi,
    X,
} from 'lucide-vue-next';
import flowIslandLogo from '../assets/flowisland-logo.svg';

const MUSIC_ONLY_DEFAULTS_KEY = 'nsd_music_only_defaults_v3';
if (localStorage.getItem(MUSIC_ONLY_DEFAULTS_KEY) !== 'true') {
    localStorage.setItem('nsd_island_opacity', '30');
    localStorage.setItem('nsd_theme_mode', 'system');
    localStorage.setItem('nsd_island_theme', 'system');
    localStorage.setItem('nsd_music_ctrl', 'true');
    localStorage.setItem('nsd_hardware_mon', 'false');
    localStorage.setItem('nsd_msg_notify', 'true');
    localStorage.setItem('nsd_msg_mode', 'true');
    localStorage.setItem('nsd_island_enabled', 'true');
    localStorage.setItem(MUSIC_ONLY_DEFAULTS_KEY, 'true');
}

type NavId = 'island' | 'music' | 'monitor' | 'about';
type IslandThemeMode = 'black' | 'white' | 'system';
type ConsoleThemeMode = 'dark' | 'light' | 'system';
type HardwareStats = {
    cpuUsage: number;
    cpuTemperature?: number | null;
    cpuFanRpm?: number | null;
    usedMemory: number;
    totalMemory: number;
    gpuUsage?: number | null;
    gpuTemperature?: number | null;
    gpuFanRpm?: number | null;
    gpuFanSpeedPercent?: number | null;
    gpuMemoryUsage?: number | null;
    gpuMemoryUsedMb?: number | null;
    gpuMemoryTotalMb?: number | null;
};

const savedIslandTheme = localStorage.getItem('nsd_island_theme');
const initialIslandTheme: IslandThemeMode = savedIslandTheme === 'black' || savedIslandTheme === 'white'
    ? savedIslandTheme
    : 'system';

const activeSection = ref<NavId>('island');
const isWidgetVisible = ref(false);
const autoStart = ref(false);
const appVersion = ref('2.3.22');
const uploadSpeed = ref('0 B/s');
const downloadSpeed = ref('0 B/s');
const opacity = ref(Number(localStorage.getItem('nsd_island_opacity') || '30'));
const lyricOffsetMs = ref(Number(localStorage.getItem('nsd_lyric_offset_ms') || '0'));
const islandTheme = ref<IslandThemeMode>(initialIslandTheme);
const themeMode = ref<ConsoleThemeMode>(
    initialIslandTheme === 'black' ? 'dark' : initialIslandTheme === 'white' ? 'light' : 'system',
);
const systemTheme = ref<'dark' | 'light'>(
    window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
);
const resolvedTheme = computed<'dark' | 'light'>(() => (
    themeMode.value === 'system' ? systemTheme.value : themeMode.value
));
const shellThemeClass = computed(() => `is-${resolvedTheme.value}`);
const enableMusicCtrl = ref(localStorage.getItem('nsd_music_ctrl') !== 'false');
const enableMsgNotify = ref(localStorage.getItem('nsd_msg_notify') !== 'false');
const enableHardwareMonitor = ref(localStorage.getItem('nsd_hardware_mon') === 'true');
const msgModeEnabled = ref(localStorage.getItem('nsd_msg_mode') !== 'false');
const pinToTaskbar = ref(localStorage.getItem('nsd_pin_taskbar') === 'true');
const isChecking = ref(false);
const isUpdating = ref(false);
const hasNewVersion = ref(false);
const updatePanelVisible = ref(false);
const updateProgress = ref(0);
const updateStatus = ref('正在准备更新…');
const updateVersion = ref('');
const updateError = ref('');
const updateDownloadedText = ref('准备下载');

const showLyrics = ref(localStorage.getItem('nsd_show_lyrics') !== 'false');
const showNetwork = ref(localStorage.getItem('nsd_show_network') !== 'false');
const showCpu = ref(localStorage.getItem('nsd_show_cpu') !== 'false');
const showGpu = ref(localStorage.getItem('nsd_show_gpu') !== 'false');
const showMemory = ref(localStorage.getItem('nsd_show_memory') !== 'false');
const monitorRefreshRate = ref(localStorage.getItem('nsd_monitor_refresh_rate') || '2');
const monitorCpuUsage = ref('--');
const monitorCpuDetail = ref('正在读取');
const monitorMemoryUsage = ref('--');
const monitorMemoryDetail = ref('正在读取');
const monitorGpuUsage = ref('--');
const monitorGpuDetail = ref('正在读取');
const monitorVramUsage = ref('--');
const monitorVramDetail = ref('正在读取');

const navItems = [
    { id: 'island' as const, label: '灵动岛', icon: Sparkles },
    { id: 'music' as const, label: '音乐与歌词', icon: Music2, badge: '实验' },
    { id: 'monitor' as const, label: '系统监控', icon: Activity },
    { id: 'about' as const, label: '关于', icon: Info },
];

const dialog = ref({
    visible: false,
    title: '',
    message: '',
    isConfirm: false,
    onConfirm: null as (() => void) | null,
});

let speedTimer: number | null = null;
let monitorTimer: number | null = null;
let lastRx = 0;
let lastTx = 0;
let lastSpeedSampleAt = 0;
let pendingUpdate: NormalizedReleaseInfo | null = null;
let unlistenUpdateProgress: (() => void) | null = null;
let unlistenWindowTheme: (() => void) | null = null;
let systemThemeMedia: MediaQueryList | null = null;

const PageHeader = defineComponent({
    props: {
        title: { type: String, required: true },
        desc: { type: String, required: true },
    },
    setup(props, { slots }) {
        return () => h('div', { class: 'page-head' }, [
            h('div', [h('h2', props.title), h('p', props.desc)]),
            slots.default?.(),
        ]);
    },
});

const SettingGroup = defineComponent({
    props: {
        title: { type: String, required: true },
        eyebrow: { type: String, required: true },
    },
    setup(props, { slots }) {
        return () => h('section', { class: 'setting-group' }, [
            h('div', { class: 'group-head' }, [h('span', props.eyebrow), h('h3', props.title)]),
            slots.default?.(),
        ]);
    },
});

const SettingRow = defineComponent({
    props: {
        title: { type: String, required: true },
        desc: { type: String, required: true },
    },
    setup(props, { slots }) {
        return () => h('div', { class: 'setting-row' }, [
            h('div', [h('strong', props.title), h('p', props.desc)]),
            h('div', { class: 'row-control' }, slots.default?.()),
        ]);
    },
});

const UiSwitch = defineComponent({
    props: {
        modelValue: { type: Boolean, required: true },
    },
    emits: ['update:modelValue', 'change'],
    setup(props, { emit: emitLocal }) {
        const onChange = (event: Event) => {
            const checked = (event.target as HTMLInputElement).checked;
            emitLocal('update:modelValue', checked);
            nextTick(() => emitLocal('change'));
        };
        return () => h('label', { class: 'ui-switch' }, [
            h('input', { type: 'checkbox', checked: props.modelValue, onChange }),
            h('span'),
        ]);
    },
});

const RangeControl = defineComponent({
    props: {
        modelValue: { type: Number, required: true },
        min: { type: [String, Number], default: 0 },
        max: { type: [String, Number], default: 100 },
        step: { type: [String, Number], default: 1 },
        suffix: { type: String, default: '%' },
    },
    emits: ['update:modelValue'],
    setup(props, { emit: emitLocal }) {
        const onInput = (event: Event) => {
            emitLocal('update:modelValue', Number((event.target as HTMLInputElement).value));
        };
        return () => h('div', { class: 'range-control' }, [
            h('input', {
                type: 'range',
                min: props.min,
                max: props.max,
                step: props.step,
                value: props.modelValue,
                onInput,
            }),
            h('strong', `${props.modelValue}${props.suffix}`),
        ]);
    },
});

const MetricCard = defineComponent({
    props: {
        icon: { type: String, required: true },
        title: { type: String, required: true },
        value: { type: String, required: true },
        detail: { type: String, required: true },
    },
    setup(props) {
        const iconByType = () => {
            if (props.icon === 'cpu') return h(Cpu, { size: 20 });
            if (props.icon === 'memory') return h(HardDrive, { size: 20 });
            if (props.icon === 'gpu') return h(Monitor, { size: 20 });
            if (props.icon === 'network') return h(Wifi, { size: 20 });
            return h(HardDrive, { size: 20 });
        };
        return () => h('article', { class: 'metric-card' }, [
            h('div', { class: 'metric-icon' }, [iconByType()]),
            h('div', [h('span', props.title), h('strong', props.value), h('p', props.detail)]),
        ]);
    },
});

const formatSpeed = (bytes: number) => {
    if (bytes < 1024) return `${bytes.toFixed(0)} B/s`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB/s`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
};

const formatOptionalPercent = (value?: number | null) =>
    typeof value === 'number' && Number.isFinite(value) ? `${Math.round(value)}%` : '--';

const formatTemperature = (value?: number | null) =>
    typeof value === 'number' && Number.isFinite(value) ? `${Math.round(value)}°C` : '--°C';

const formatFan = (rpm?: number | null, percent?: number | null) => {
    if (typeof rpm === 'number' && Number.isFinite(rpm)) return `${Math.round(rpm)}RPM`;
    if (typeof percent === 'number' && Number.isFinite(percent)) return `${Math.round(percent)}%`;
    return '风扇 --';
};

const formatMemory = (usedBytes: number, totalBytes: number) => {
    if (!Number.isFinite(usedBytes) || !Number.isFinite(totalBytes) || totalBytes <= 0) return '-- / --';
    return `${(usedBytes / 1024 / 1024 / 1024).toFixed(1)} / ${(totalBytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
};

const formatVram = (usedMb?: number | null, totalMb?: number | null) => {
    if (typeof usedMb !== 'number' || typeof totalMb !== 'number' || totalMb <= 0) return '-- / --';
    return `${(usedMb / 1024).toFixed(1)} / ${(totalMb / 1024).toFixed(1)} GB`;
};

const fetchHardwareStats = async () => {
    try {
        const stats = await invoke<HardwareStats>('get_hardware_stats');
        monitorCpuUsage.value = formatOptionalPercent(stats.cpuUsage);
        monitorCpuDetail.value = `${formatTemperature(stats.cpuTemperature)} · ${formatFan(stats.cpuFanRpm)}`;
        monitorMemoryUsage.value = stats.totalMemory > 0
            ? `${Math.round((stats.usedMemory / stats.totalMemory) * 100)}%`
            : '--';
        monitorMemoryDetail.value = formatMemory(stats.usedMemory, stats.totalMemory);
        monitorGpuUsage.value = formatOptionalPercent(stats.gpuUsage);
        monitorGpuDetail.value = `${formatTemperature(stats.gpuTemperature)} · ${formatFan(stats.gpuFanRpm, stats.gpuFanSpeedPercent)}`;
        monitorVramUsage.value = formatOptionalPercent(stats.gpuMemoryUsage);
        monitorVramDetail.value = formatVram(stats.gpuMemoryUsedMb, stats.gpuMemoryTotalMb);
    } catch {
        monitorCpuDetail.value = '当前环境无法读取';
        monitorMemoryDetail.value = '当前环境无法读取';
        monitorGpuDetail.value = '当前环境无法读取';
        monitorVramDetail.value = '当前环境无法读取';
    }
};

const restartMonitorTimer = () => {
    if (monitorTimer != null) {
        window.clearInterval(monitorTimer);
        monitorTimer = null;
    }
    if (!enableHardwareMonitor.value && activeSection.value !== 'monitor') return;
    const refreshMs = Math.max(1, Number(monitorRefreshRate.value) || 2) * 1000;
    monitorTimer = window.setInterval(fetchHardwareStats, refreshMs);
};

const fetchSpeedStats = async () => {
    try {
        const [currentRx, currentTx] = await invoke<[number, number]>('get_network_stats');
        const sampledAt = performance.now();
        if (lastRx !== 0 && lastTx !== 0) {
            const elapsedSeconds = Math.max(0.25, (sampledAt - lastSpeedSampleAt) / 1000);
            downloadSpeed.value = formatSpeed(Math.max(0, (currentRx - lastRx) / elapsedSeconds));
            uploadSpeed.value = formatSpeed(Math.max(0, (currentTx - lastTx) / elapsedSeconds));
        }
        lastRx = currentRx;
        lastTx = currentTx;
        lastSpeedSampleAt = sampledAt;
    } catch (error) {
        console.error('获取网络速度失败:', error);
    }
};

const toggleWidget = async () => {
    const nextState = !isWidgetVisible.value;
    await emit('control-island-visibility', { show: nextState });
    isWidgetVisible.value = nextState;
};

const toggleAutoStart = async () => {
    try {
        if (autoStart.value) {
            await enable();
        } else {
            await disable();
        }
    } catch (error) {
        autoStart.value = !autoStart.value;
        showDialog('设置失败', '开机自启动设置失败，请检查系统权限。');
    }
};

const togglePinTaskbar = async () => {
    localStorage.setItem('nsd_pin_taskbar', String(pinToTaskbar.value));
    await emit('control-pin-taskbar', { enabled: pinToTaskbar.value });
};

const toggleMsgMode = async () => {
    localStorage.setItem('nsd_msg_mode', String(msgModeEnabled.value));
    await emit('control-msg-mode', { enabled: msgModeEnabled.value });
};

const toggleMsgNotify = () => {
    localStorage.setItem('nsd_msg_notify', String(enableMsgNotify.value));
};

const applyWindowTheme = async (mode: ConsoleThemeMode) => {
    try {
        const win = getCurrentWindow();
        if (mode === 'system') {
            await win.setTheme(null);
            const currentTheme = await win.theme();
            if (currentTheme === 'dark' || currentTheme === 'light') {
                systemTheme.value = currentTheme;
            }
        } else {
            await win.setTheme(mode);
        }
    } catch {
        if (mode === 'system') {
            systemTheme.value = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
    }
};

const handleSystemThemeChange = (event: MediaQueryListEvent) => {
    systemTheme.value = event.matches ? 'dark' : 'light';
};

const minimizeWindow = async () => {
    await getCurrentWindow().minimize();
};

const closeWindow = async () => {
    await getCurrentWindow().hide();
};

const showDialog = (title: string, message: string, isConfirm = false, onConfirm: (() => void) | null = null) => {
    dialog.value = { visible: true, title, message, isConfirm, onConfirm };
};

const openProjectRepo = () => {
    openUrl('https://github.com/CHmua/FlowIsland');
};

const closeDialog = () => {
    dialog.value.visible = false;
};

const handleDialogConfirm = () => {
    dialog.value.onConfirm?.();
    closeDialog();
};

const parseVersion = (value: string) => {
    const match = value.match(/\d+\.\d+\.\d+/);
    return match ? match[0].split('.').map(Number) : [0, 0, 0];
};

type UpdateReleaseInfo = {
    tagName?: string;
    tag_name?: string;
    name?: string;
    htmlUrl?: string;
    html_url?: string;
    downloadUrl?: string | null;
    download_url?: string | null;
    assetDigest?: string | null;
    asset_digest?: string | null;
    assetSize?: number | null;
    asset_size?: number | null;
    source?: string;
    fetchedAt?: number;
};

type NormalizedReleaseInfo = {
    tagName: string;
    name: string;
    htmlUrl: string;
    downloadUrl: string;
    assetDigest: string | null;
    assetSize: number | null;
    source: string;
    fetchedAt: number;
};

const UPDATE_CACHE_KEY = 'flowisland_update_cache_v1';
const UPDATE_CACHE_TTL_MS = 5 * 60 * 1000;
const UPDATE_FALLBACK_CACHE_TTL_MS = 24 * 60 * 60 * 1000;

const compareVersions = (remote: number[], local: number[]) => {
    for (let index = 0; index < Math.max(remote.length, local.length); index += 1) {
        const remotePart = remote[index] || 0;
        const localPart = local[index] || 0;
        if (remotePart > localPart) return 1;
        if (remotePart < localPart) return -1;
    }
    return 0;
};

const normalizeReleaseInfo = (raw: UpdateReleaseInfo | null | undefined): NormalizedReleaseInfo | null => {
    if (!raw) return null;
    const tagName = raw.tagName || raw.tag_name || raw.name || '';
    if (!tagName.trim()) return null;
    return {
        tagName,
        name: raw.name || tagName,
        htmlUrl: raw.htmlUrl || raw.html_url || 'https://github.com/CHmua/FlowIsland/releases/latest',
        downloadUrl: raw.downloadUrl || raw.download_url || '',
        assetDigest: raw.assetDigest || raw.asset_digest || null,
        assetSize: raw.assetSize || raw.asset_size || null,
        source: raw.source || 'unknown',
        fetchedAt: raw.fetchedAt || Date.now(),
    };
};

const readCachedReleaseInfo = (maxAgeMs: number) => {
    try {
        const cached = normalizeReleaseInfo(JSON.parse(localStorage.getItem(UPDATE_CACHE_KEY) || 'null'));
        if (!cached) return null;
        return Date.now() - cached.fetchedAt <= maxAgeMs ? cached : null;
    } catch {
        return null;
    }
};

const writeCachedReleaseInfo = (release: NormalizedReleaseInfo) => {
    localStorage.setItem(UPDATE_CACHE_KEY, JSON.stringify({ ...release, fetchedAt: Date.now() }));
};

const formatCacheAge = (fetchedAt: number) => {
    const minutes = Math.max(0, Math.round((Date.now() - fetchedAt) / 60000));
    if (minutes <= 0) return '刚刚';
    if (minutes < 60) return `${minutes} 分钟前`;
    return `${Math.round(minutes / 60)} 小时前`;
};

type UpdateProgressPayload = {
    status: string;
    downloaded: number;
    total?: number | null;
    percent: number;
};

const formatBytes = (bytes: number) => {
    if (!Number.isFinite(bytes) || bytes <= 0) return '准备下载';
    if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${Math.max(1, Math.round(bytes / 1024))} KB`;
};

const startUpdate = async (release: NormalizedReleaseInfo) => {
    if (isUpdating.value) return;
    if (!release.downloadUrl) {
        showDialog('暂时无法更新', '新版本尚未上传 Windows 安装包，请稍后再试。');
        return;
    }

    pendingUpdate = release;
    isUpdating.value = true;
    updatePanelVisible.value = true;
    updateProgress.value = 0;
    updateStatus.value = '正在连接 GitHub…';
    updateVersion.value = release.tagName;
    updateError.value = '';
    updateDownloadedText.value = '准备下载';

    try {
        await invoke('download_and_install_update', {
            downloadUrl: release.downloadUrl,
            version: release.tagName,
            expectedDigest: release.assetDigest,
            expectedSize: release.assetSize,
        });
    } catch (error) {
        isUpdating.value = false;
        updateError.value = error instanceof Error ? error.message : String(error || '无法下载或安装更新。');
    }
};

const retryUpdate = () => {
    if (pendingUpdate) void startUpdate(pendingUpdate);
};

const closeUpdatePanel = () => {
    if (isUpdating.value) return;
    updatePanelVisible.value = false;
    updateError.value = '';
};

const showUpdateResult = (release: NormalizedReleaseInfo, fromCache = false, prefix = '') => {
    const local = parseVersion(appVersion.value);
    const remote = parseVersion(release.tagName || release.name || '');
    const hasUpdate = compareVersions(remote, local) > 0;
    hasNewVersion.value = hasUpdate;
    const cacheText = fromCache ? `\n\n结果来自 ${formatCacheAge(release.fetchedAt)} 的本地缓存。` : '';
    const sourceText = release.source === 'github-page' ? '\n\n已使用 GitHub Releases 页面兜底检查。' : '';
    if (hasUpdate) {
        showDialog(
            '发现新版本',
            `${prefix}检测到新版本 ${release.tagName}。\n\n点击“确定”后将在软件内下载并自动安装，安装时 FlowIsland 会短暂退出。${sourceText}${cacheText}`,
            true,
            () => {
                void startUpdate(release);
            }
        );
        return;
    }

    showDialog('已是最新版本', `${prefix}当前已经是最新版本。${sourceText}${cacheText}`);
};

const checkUpdate = async () => {
    if (isChecking.value || isUpdating.value) return;
    const freshCache = readCachedReleaseInfo(UPDATE_CACHE_TTL_MS);
    if (freshCache) {
        showUpdateResult(freshCache, true);
        return;
    }

    isChecking.value = true;
    try {
        const release = normalizeReleaseInfo(await invoke<UpdateReleaseInfo>('fetch_latest_release_info'));
        if (!release) throw new Error('更新源没有返回有效版本号。');
        writeCachedReleaseInfo(release);
        showUpdateResult(release);
    } catch (error) {
        const fallbackCache = readCachedReleaseInfo(UPDATE_FALLBACK_CACHE_TTL_MS);
        if (fallbackCache) {
            showUpdateResult(
                fallbackCache,
                true,
                '暂时无法连接 GitHub，已使用最近一次缓存结果。\n\n'
            );
        } else {
            showDialog('检查失败', error instanceof Error ? error.message : '无法检查更新，请稍后再试。');
        }
    } finally {
        isChecking.value = false;
    }
};

watch(opacity, async (value) => {
    const next = Math.max(0, Math.min(100, Number(value) || 0));
    if (next !== value) {
        opacity.value = next;
        return;
    }
    localStorage.setItem('nsd_island_opacity', String(next));
    await emit('control-island-opacity', { opacity: next });
});

watch(lyricOffsetMs, async (value) => {
    const next = Math.max(-3000, Math.min(3000, Number(value) || 0));
    if (next !== value) {
        lyricOffsetMs.value = next;
        return;
    }
    localStorage.setItem('nsd_lyric_offset_ms', String(next));
    await emit('control-lyric-offset', { offsetMs: next });
});

watch(islandTheme, async (value) => {
    const nextMode: ConsoleThemeMode = value === 'black' ? 'dark' : value === 'white' ? 'light' : 'system';
    themeMode.value = nextMode;
    localStorage.setItem('nsd_theme_mode', nextMode);
    localStorage.setItem('nsd_island_theme', value);
    await applyWindowTheme(nextMode);
    await emit('control-island-theme', { theme: value });
});

watch(enableMusicCtrl, async (value) => {
    localStorage.setItem('nsd_music_ctrl', String(value));
    await emit('control-music-ctl', { enabled: value });
});

watch(showLyrics, async (value) => {
    localStorage.setItem('nsd_show_lyrics', String(value));
    await emit('control-lyrics-visibility', { enabled: value });
});

watch(enableHardwareMonitor, async (value) => {
    localStorage.setItem('nsd_hardware_mon', String(value));
    await emit('control-hardware-mon', { enabled: value });
    if (value) await fetchHardwareStats();
    restartMonitorTimer();
});

watch([showCpu, showGpu, showMemory, showNetwork], async ([cpu, gpu, memory, network]) => {
    localStorage.setItem('nsd_show_cpu', String(cpu));
    localStorage.setItem('nsd_show_gpu', String(gpu));
    localStorage.setItem('nsd_show_memory', String(memory));
    localStorage.setItem('nsd_show_network', String(network));
    await emit('control-monitor-metrics', { cpu, gpu, memory, network });
});

watch(monitorRefreshRate, (value) => {
    localStorage.setItem('nsd_monitor_refresh_rate', value);
    restartMonitorTimer();
});

watch(activeSection, async (section) => {
    if (section === 'monitor') await fetchHardwareStats();
    restartMonitorTimer();
});

onMounted(async () => {
    systemThemeMedia = window.matchMedia('(prefers-color-scheme: dark)');
    systemTheme.value = systemThemeMedia.matches ? 'dark' : 'light';
    systemThemeMedia.addEventListener('change', handleSystemThemeChange);

    await applyWindowTheme(themeMode.value);
    try {
        const win = getCurrentWindow();
        unlistenWindowTheme = await win.onThemeChanged(({ payload }) => {
            if (payload === 'dark' || payload === 'light') {
                systemTheme.value = payload;
            }
        });
    } catch {
        // Browser preview falls back to prefers-color-scheme.
    }

    try {
        appVersion.value = await getVersion();
    } catch { /* ignore */ }

    try {
        autoStart.value = await isEnabled();
    } catch { /* ignore */ }

    await fetchSpeedStats();
    speedTimer = window.setInterval(fetchSpeedStats, 2000);
    if (enableHardwareMonitor.value || activeSection.value === 'monitor') {
        await fetchHardwareStats();
    }
    restartMonitorTimer();

    try {
        await listen('open-settings-panel', async () => {
            activeSection.value = 'island';
            const win = getCurrentWindow();
            await win.show();
            await win.unminimize();
            await win.setFocus();
        });

        await listen<{ visible: boolean }>('island-status-sync', (event) => {
            isWidgetVisible.value = event.payload.visible;
        });

        unlistenUpdateProgress = await listen<UpdateProgressPayload>('update-download-progress', (event) => {
            const progress = event.payload;
            updatePanelVisible.value = true;
            updateStatus.value = progress.status;
            updateProgress.value = Math.max(0, Math.min(100, Math.round(progress.percent || 0)));
            const downloaded = formatBytes(progress.downloaded || 0);
            updateDownloadedText.value = progress.total
                ? `${downloaded} / ${formatBytes(progress.total)}`
                : downloaded;
        });
    } catch {
        // Browser-only preview does not provide the Tauri event bridge.
    }

    for (let i = 0; i < 6; i += 1) {
        try {
            const visible = await invoke<boolean>('is_widget_visible');
            if (visible) {
                isWidgetVisible.value = true;
                return;
            }
        } catch { /* ignore */ }
        await new Promise((resolve) => setTimeout(resolve, 180));
    }
});

onUnmounted(() => {
    if (speedTimer != null) {
        window.clearInterval(speedTimer);
    }
    if (monitorTimer != null) {
        window.clearInterval(monitorTimer);
    }
    systemThemeMedia?.removeEventListener('change', handleSystemThemeChange);
    systemThemeMedia = null;
    unlistenWindowTheme?.();
    unlistenWindowTheme = null;
    unlistenUpdateProgress?.();
    unlistenUpdateProgress = null;
});
</script>

<style>
*,
*::before,
*::after {
    box-sizing: border-box;
}

:global(html),
:global(body),
:global(#app) {
    width: 100%;
    height: 100%;
    margin: 0;
    overflow: hidden;
    background: transparent;
}

:global(body) {
    font-family: "Microsoft YaHei UI", "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
    color: #f5f7fa;
    user-select: none;
    -webkit-font-smoothing: antialiased;
}

button,
input,
select {
    font: inherit;
}

button {
    border: 0;
    cursor: pointer;
}

.fi-shell {
    width: 100%;
    height: 100vh;
    overflow: hidden;
    background:
        radial-gradient(circle at 20% 8%, rgba(76, 141, 255, 0.16), transparent 28%),
        radial-gradient(circle at 82% 15%, rgba(69, 199, 232, 0.10), transparent 30%),
        linear-gradient(145deg, #0c111b, #0a0f17 62%);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: #f5f7fa;
}

.fi-titlebar {
    height: 44px;
    display: grid;
    grid-template-columns: 1fr auto auto;
    align-items: center;
    padding: 0 14px 0 18px;
    background: rgba(13, 18, 28, 0.86);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.title-brand,
.title-status,
.window-actions,
.brand-block,
.nav-item,
.update-link,
.head-toggle,
.setting-row,
.color-strip,
.check-item,
.preview-head,
.live-dot,
.preview-tabs,
.preview-note,
.app-source,
.about-card,
.about-actions,
.ghost-btn,
.secondary-btn {
    display: flex;
    align-items: center;
}

.title-brand {
    gap: 9px;
    color: #eef4ff;
    font-size: 13px;
}

.brand-mark,
.logo-mark {
    display: grid;
    place-items: center;
    overflow: hidden;
    background: transparent;
    border: 0;
}

.brand-mark img,
.logo-mark img {
    display: block;
    width: 100%;
    height: 100%;
}

.brand-mark {
    width: 26px;
    height: 26px;
    border-radius: 9px;
}

.title-status {
    gap: 7px;
    color: #9fb0c8;
    font-size: 12px;
    padding: 6px 10px;
    border-radius: 999px;
    background: rgba(76, 141, 255, 0.08);
}

.title-status i,
.live-dot i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #4cd7a0;
    box-shadow: 0 0 12px rgba(76, 215, 160, 0.55);
}

.live-dot i.is-muted,
.title-status:has(span:first-child:last-child) i {
    background: #67758a;
    box-shadow: none;
}

.window-actions {
    gap: 8px;
    margin-left: 18px;
}

.window-actions button {
    width: 30px;
    height: 24px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: #91a0b8;
    background: transparent;
}

.window-actions button:hover {
    color: #f5f7fa;
    background: rgba(255, 255, 255, 0.08);
}

.window-actions button:last-child:hover {
    background: rgba(240, 107, 114, 0.78);
}

.fi-body {
    height: calc(100% - 44px);
    display: grid;
    grid-template-columns: 190px minmax(0, 1fr);
}

.sidebar {
    min-width: 0;
    background: linear-gradient(180deg, #0e1420, #0b111b);
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    padding: 18px 14px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.brand-block {
    gap: 12px;
    padding: 8px 6px 14px;
}

.logo-mark {
    width: 46px;
    height: 46px;
    border-radius: 15px;
    flex: 0 0 auto;
}

.brand-block h1 {
    margin: 0;
    font-size: 16px;
    line-height: 1.1;
    font-weight: 650;
}

.brand-block p {
    margin: 4px 0 0;
    color: #6fa9ff;
    font-size: 11px;
}

.nav-list {
    display: grid;
    gap: 5px;
}

.nav-item {
    width: 100%;
    height: 38px;
    gap: 10px;
    padding: 0 10px;
    color: #9aa8bc;
    background: transparent;
    border-radius: 11px;
    text-align: left;
    position: relative;
}

.nav-item span {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    font-weight: 500;
}

.nav-item em {
    font-style: normal;
    color: #f4b85a;
    border: 1px solid rgba(244, 184, 90, 0.35);
    border-radius: 7px;
    padding: 1px 5px;
    font-size: 10px;
}

.nav-item:hover {
    color: #eef4ff;
    background: rgba(76, 141, 255, 0.08);
}

.nav-item.is-active {
    color: #f5f7fa;
    background: rgba(76, 141, 255, 0.16);
}

.nav-item.is-active::before {
    content: "";
    width: 4px;
    height: 18px;
    border-radius: 999px;
    background: #4c8dff;
    position: absolute;
    left: -3px;
}

.update-link {
    gap: 8px;
    margin-top: auto;
    height: 36px;
    border-radius: 11px;
    justify-content: center;
    color: #b9c7d9;
    background: rgba(255, 255, 255, 0.045);
}

.update-link:hover {
    color: #f5f7fa;
    background: rgba(76, 141, 255, 0.12);
}

.main-content {
    min-width: 0;
    overflow: hidden;
    padding: 22px 24px 24px;
}

.page-layout {
    height: 100%;
    min-height: 0;
}

.page-layout.has-preview {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 22px;
}

.single-page {
    overflow-y: auto;
    padding-right: 4px;
}

.settings-pane {
    min-width: 0;
    overflow-y: auto;
    padding-right: 4px;
}

.settings-pane.full {
    max-width: 100%;
}

.settings-pane::-webkit-scrollbar,
.single-page::-webkit-scrollbar {
    width: 6px;
}

.settings-pane::-webkit-scrollbar-thumb,
.single-page::-webkit-scrollbar-thumb {
    border-radius: 999px;
    background: rgba(154, 168, 188, 0.24);
}

.page-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 18px;
    margin-bottom: 18px;
}

.page-head h2 {
    margin: 0;
    font-size: 24px;
    line-height: 1.2;
    font-weight: 650;
    letter-spacing: 0;
}

.page-head p {
    margin: 7px 0 0;
    color: #9aa8bc;
    font-size: 12px;
}

.head-toggle {
    gap: 10px;
    flex: 0 0 auto;
    color: #b9c7d9;
    font-size: 13px;
    background: rgba(76, 141, 255, 0.09);
    border-radius: 12px;
    padding: 8px 10px 8px 12px;
}

.setting-group {
    background: #121a28;
    border-radius: 16px;
    margin-bottom: 14px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.045);
}

.category-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
    align-items: stretch;
}

.category-grid .setting-group {
    margin-bottom: 0;
}

.category-grid .setting-group:last-child {
    grid-column: 1 / -1;
}

.group-head {
    padding: 15px 18px 8px;
}

.group-head span {
    display: block;
    color: #4c8dff;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
}

.group-head h3 {
    margin: 4px 0 0;
    font-size: 15px;
    font-weight: 600;
}

.setting-row {
    min-height: 64px;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 18px;
    border-top: 1px solid rgba(255, 255, 255, 0.045);
}

.setting-row > div:first-child {
    min-width: 0;
}

.setting-row strong {
    display: block;
    color: #eef4ff;
    font-size: 14px;
    font-weight: 550;
}

.setting-row p {
    margin: 5px 0 0;
    color: #8392a7;
    font-size: 12px;
    line-height: 1.45;
}

.row-control {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: flex-end;
}

.ui-switch,
.native-switch {
    width: 42px;
    height: 24px;
    padding: 3px;
    border-radius: 999px;
    background: #334052;
    flex: 0 0 auto;
    transition: background 0.2s ease;
    display: inline-block;
}

.ui-switch input,
.native-switch input {
    width: 0;
    height: 0;
    opacity: 0;
    position: absolute;
}

.ui-switch span,
.native-switch span {
    display: block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #e9edf3;
    transition: transform 0.2s ease;
}

.ui-switch:has(input:checked),
.native-switch:has(input:checked) {
    background: #2f73f6;
}

.ui-switch input:checked + span,
.native-switch input:checked + span {
    transform: translateX(18px);
}

.segmented {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 12px;
    background: #172131;
    flex: 0 0 auto;
}

.segmented.small {
    border-radius: 10px;
}

.segmented button,
.preview-tab {
    height: 30px;
    padding: 0 14px;
    border-radius: 9px;
    color: #9aa8bc;
    background: transparent;
    font-size: 12px;
    font-weight: 550;
}

.segmented button.is-active,
.preview-tab.is-active {
    color: #f5f7fa;
    background: #2f73f6;
}

.color-strip {
    gap: 10px;
    padding: 8px 18px 14px;
}

.color-strip.compact {
    padding: 0;
}

.swatch {
    width: 28px;
    height: 28px;
    border-radius: 10px;
    background: var(--swatch);
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.14);
    position: relative;
}

.swatch.is-active::after {
    content: "";
    position: absolute;
    inset: -4px;
    border-radius: 13px;
    border: 2px solid rgba(76, 141, 255, 0.65);
}

.swatch.custom {
    color: #d6e3f4;
    background: #172131;
    font-size: 18px;
}

.range-control {
    width: 188px;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 58px;
    align-items: center;
    gap: 10px;
}

.range-control input {
    width: 100%;
    accent-color: #4c8dff;
}

.range-control strong {
    color: #d6e3f4;
    font-size: 12px;
    text-align: right;
}

.check-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    padding: 10px 18px 18px;
}

.check-item {
    gap: 8px;
    height: 34px;
    color: #c7d3e2;
    font-size: 12px;
    background: rgba(255, 255, 255, 0.035);
    border-radius: 10px;
    padding: 0 10px;
}

.check-item input {
    accent-color: #4c8dff;
}

.position-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 9px;
    padding: 10px 18px 4px;
}

.position-card {
    height: 74px;
    border-radius: 12px;
    color: #aebbd0;
    background: #172131;
    display: grid;
    place-items: center;
    gap: 6px;
    font-size: 12px;
}

.position-card i {
    width: 52px;
    height: 30px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 7px;
    position: relative;
}

.position-card i::before {
    content: "";
    position: absolute;
    top: 5px;
    left: 19px;
    width: 14px;
    height: 4px;
    border-radius: 999px;
    background: #4c8dff;
}

.position-card.is-active {
    color: #f5f7fa;
    background: rgba(76, 141, 255, 0.16);
}

.secondary-btn,
.ghost-btn,
.primary-btn,
.danger-btn,
.text-button {
    min-height: 32px;
    padding: 0 14px;
    border-radius: 10px;
    font-size: 12px;
    font-weight: 550;
    gap: 7px;
}

.secondary-btn {
    color: #dbe8fb;
    background: #1c283a;
}

.ghost-btn {
    color: #9fb0c8;
    background: transparent;
}

.primary-btn {
    color: #fff;
    background: #2f73f6;
}

.danger-btn {
    color: #fff;
    background: rgba(240, 107, 114, 0.82);
}

.text-button {
    color: #8fc1ff;
    background: rgba(76, 141, 255, 0.1);
}

.select {
    height: 32px;
    border: 0;
    border-radius: 10px;
    color: #d6e3f4;
    background: #1c283a;
    padding: 0 32px 0 12px;
}

.preview-panel {
    height: 100%;
    background: linear-gradient(180deg, #121a28, #0f1724);
    border-radius: 18px;
    border: 1px solid rgba(255, 255, 255, 0.055);
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
}

.preview-head {
    justify-content: space-between;
    gap: 14px;
}

.preview-head span,
.side-preview-card > span,
.theme-preview > span {
    color: #7f8da3;
    font-size: 12px;
}

.preview-head h3 {
    margin: 5px 0 0;
    font-size: 18px;
    font-weight: 650;
}

.live-dot {
    gap: 7px;
    color: #9fb0c8;
    font-size: 12px;
}

.preview-stage {
    flex: 1;
    min-height: 260px;
}

.desktop-frame {
    height: 100%;
    min-height: 292px;
    border-radius: 16px;
    overflow: hidden;
    position: relative;
    background:
        linear-gradient(160deg, rgba(76, 141, 255, 0.24), transparent 42%),
        linear-gradient(25deg, rgba(69, 199, 232, 0.10), transparent 50%),
        #07101d;
}

.wallpaper-glow {
    position: absolute;
    inset: 0;
    background:
        radial-gradient(circle at 56% 32%, rgba(76, 141, 255, 0.30), transparent 34%),
        radial-gradient(circle at 35% 70%, rgba(69, 199, 232, 0.14), transparent 28%);
    filter: blur(10px);
}

.taskbar-preview {
    position: absolute;
    left: 18px;
    right: 18px;
    bottom: 18px;
    height: 48px;
    border-radius: 15px;
    background: rgba(11, 15, 23, 0.78);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 14px;
    backdrop-filter: blur(18px);
}

.taskbar-preview span {
    width: 22px;
    height: 22px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.14);
}

.taskbar-preview strong {
    margin-left: auto;
    color: #dce8f8;
    font-size: 12px;
}

.mock-island {
    position: absolute;
    left: 50%;
    top: 52px;
    transform: translateX(-50%);
    min-height: 38px;
    border-radius: 999px;
    padding: 5px 9px;
    background:
        linear-gradient(90deg, rgba(76, 141, 255, 0.24), rgba(69, 199, 232, 0.16), rgba(244, 184, 90, 0.16)),
        rgba(12, 16, 24, 0.72);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08), 0 12px 30px rgba(0, 0, 0, 0.22);
    backdrop-filter: blur(18px);
    display: flex;
    align-items: center;
    gap: 9px;
    z-index: 2;
    white-space: nowrap;
}

.mock-default {
    width: 246px;
    justify-content: center;
}

.mock-music,
.mock-lyric {
    width: 286px;
}

.mock-notify,
.mock-task,
.mock-panda {
    width: 276px;
}

.mock-monitor {
    width: 330px;
    justify-content: space-around;
    font-size: 12px;
    color: #d9e4f4;
}

.mock-monitor span {
    display: flex;
    align-items: center;
    gap: 5px;
}

.mock-monitor strong {
    color: #fff;
    font-size: 13px;
}

.time-pill,
.speed-pill {
    color: #f3f7ff;
    font-size: 12px;
    font-weight: 600;
}

.speed-pill {
    color: #c8d4e5;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #4cd7a0;
}

.cover-art,
.app-dot,
.panda-face {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    flex: 0 0 auto;
}

.cover-art {
    background:
        radial-gradient(circle at 36% 35%, #fff, transparent 18%),
        linear-gradient(135deg, #84d8ff, #4c8dff 48%, #1c2e54);
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.78);
}

.lyric-cover {
    background:
        radial-gradient(circle at 42% 42%, #fff, transparent 16%),
        linear-gradient(135deg, #f4b85a, #8b6dff 46%, #2f153f);
}

.track-copy,
.lyric-copy,
.notify-copy {
    min-width: 0;
    flex: 1;
}

.track-copy strong,
.lyric-copy strong,
.notify-copy strong {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #fff;
    font-size: 12px;
    font-weight: 650;
}

.track-copy small,
.lyric-copy small,
.notify-copy small {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #aebbd0;
    font-size: 10px;
    margin-top: 1px;
}

.progress-line {
    position: absolute;
    left: 48px;
    right: 54px;
    bottom: 3px;
    height: 2px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.14);
}

.progress-line i,
.mini-meter i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: #45c7e8;
}

.round-play {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: #f6fbff;
    background: rgba(255, 255, 255, 0.12);
}

.audio-wave {
    display: flex;
    align-items: center;
    gap: 2px;
    width: 18px;
}

.audio-wave i {
    width: 2px;
    border-radius: 999px;
    background: #45c7e8;
    animation: wave 0.9s ease-in-out infinite;
}

.audio-wave .bar-1 { height: 7px; animation-delay: 0s; }
.audio-wave .bar-2 { height: 14px; animation-delay: 0.12s; }
.audio-wave .bar-3 { height: 10px; animation-delay: 0.24s; }
.audio-wave .bar-4 { height: 16px; animation-delay: 0.36s; }

.app-dot {
    display: grid;
    place-items: center;
    color: #d8ecff;
    background: rgba(76, 141, 255, 0.20);
}

.mini-meter {
    width: 52px;
    height: 4px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.14);
}

.panda-face {
    position: relative;
    background: #f6fbff;
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.5);
}

.panda-face::before,
.panda-face::after {
    content: "";
    position: absolute;
    top: 4px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #182235;
}

.panda-face::before { left: 4px; }
.panda-face::after { right: 4px; }

.panda-face i,
.panda-face b {
    position: absolute;
    top: -3px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #182235;
}

.panda-face i { left: 1px; }
.panda-face b { right: 1px; }

.preview-tabs {
    gap: 6px;
    flex-wrap: wrap;
}

.preview-tab {
    background: #172131;
    padding: 0 11px;
}

.preview-note {
    gap: 9px;
    align-items: flex-start;
    padding: 12px;
    border-radius: 14px;
    color: #78b5ff;
    background: rgba(76, 141, 255, 0.08);
}

.preview-note p {
    margin: 0;
    color: #9aa8bc;
    font-size: 12px;
    line-height: 1.5;
}

.two-column-page {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 290px;
    gap: 18px;
}

.side-preview-card,
.theme-preview {
    min-height: 240px;
    background: #121a28;
    border-radius: 16px;
    padding: 18px;
    border: 1px solid rgba(255, 255, 255, 0.045);
}

.side-preview-card {
    position: relative;
    overflow: hidden;
}

.side-preview-card .mock-island {
    position: relative;
    top: auto;
    left: auto;
    transform: none;
    margin: 24px 0 20px;
    width: 100%;
    max-width: 100%;
}

.side-preview-card p {
    color: #9aa8bc;
    font-size: 12px;
    line-height: 1.6;
}

.app-list {
    padding: 4px 0 8px;
}

.app-source {
    gap: 12px;
    padding: 11px 18px;
    border-top: 1px solid rgba(255, 255, 255, 0.045);
}

.app-source > span {
    width: 32px;
    height: 32px;
    border-radius: 10px;
    background: #172131;
    display: grid;
    place-items: center;
    color: #9ec8ff;
    font-weight: 700;
}

.app-source div {
    flex: 1;
}

.app-source strong {
    display: block;
    font-size: 13px;
}

.app-source p,
.lyric-stack p {
    margin: 3px 0 0;
    color: #8392a7;
    font-size: 11px;
}

.status-pill {
    border-radius: 999px;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 650;
}

.status-pill.ok {
    color: #7ff0bd;
    background: rgba(76, 215, 160, 0.12);
}

.lyric-stack {
    display: grid;
    gap: 10px;
    padding-top: 8px;
}

.lyric-stack strong {
    color: #fff;
    font-size: 16px;
}

.metric-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    margin-bottom: 14px;
}

.metric-card {
    min-height: 104px;
    border-radius: 16px;
    padding: 16px;
    background: #121a28;
    border: 1px solid rgba(255, 255, 255, 0.045);
    overflow: hidden;
    position: relative;
}

.metric-icon {
    width: 34px;
    height: 34px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    color: #8fc1ff;
    background: rgba(76, 141, 255, 0.12);
    margin-bottom: 8px;
}

.metric-card span {
    color: #9aa8bc;
    font-size: 12px;
}

.metric-card strong {
    display: block;
    margin-top: 6px;
    font-size: 22px;
    font-weight: 650;
}

.metric-card p {
    margin: 4px 0 0;
    color: #8392a7;
    font-size: 11px;
}

.theme-preview {
    display: grid;
    align-content: start;
    gap: 18px;
}

.mini-window {
    height: 250px;
    border-radius: 16px;
    background: linear-gradient(145deg, #0d1524, #141f31);
    padding: 18px;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
}

.mini-window i {
    height: 76px;
    border-radius: 14px;
    background: rgba(76, 141, 255, 0.14);
}

.mini-window div {
    grid-column: span 3;
    height: 26px;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.08);
}

.about-card {
    gap: 16px;
    padding: 20px;
    border-radius: 18px;
    background: #121a28;
    border: 1px solid rgba(255, 255, 255, 0.045);
    margin-bottom: 14px;
}

.about-logo {
    width: 62px;
    height: 62px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: transparent;
}

.about-logo img {
    display: block;
    width: 100%;
    height: 100%;
}

.about-card h3 {
    margin: 0;
    font-size: 22px;
}

.about-card p {
    margin: 5px 0 0;
    color: #9aa8bc;
}

.about-actions {
    margin-left: auto;
    gap: 8px;
}

.modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(2, 6, 23, 0.66);
    backdrop-filter: blur(8px);
}

.modal-card {
    width: 360px;
    padding: 24px;
    border-radius: 20px;
    border: 1px solid rgba(109, 137, 183, 0.24);
    background: #0c1423;
    box-shadow: 0 24px 70px rgba(0, 0, 0, 0.42);
}

.modal-card h3 {
    margin: 0;
    font-size: 20px;
}

.modal-card p {
    margin: 14px 0 0;
    color: rgba(203, 213, 225, 0.78);
    line-height: 1.6;
    white-space: pre-line;
}

.modal-actions {
    margin-top: 22px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
}

.update-modal {
    width: 390px;
    text-align: center;
}

.update-modal-icon {
    width: 52px;
    height: 52px;
    margin: 0 auto 15px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    color: #b8d7ff;
    background: rgba(76, 141, 255, 0.16);
    box-shadow: inset 0 0 0 1px rgba(112, 166, 255, 0.24);
}

.update-modal-icon.has-error {
    color: #ffb4b4;
    background: rgba(239, 92, 92, 0.14);
    box-shadow: inset 0 0 0 1px rgba(239, 92, 92, 0.22);
}

.update-progress {
    height: 7px;
    margin-top: 20px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
}

.update-progress i {
    display: block;
    height: 100%;
    min-width: 3px;
    border-radius: inherit;
    background: linear-gradient(90deg, #3f7fff, #53c8ff);
    box-shadow: 0 0 14px rgba(76, 141, 255, 0.42);
    transition: width 0.18s ease;
}

.update-progress-meta {
    display: flex;
    justify-content: space-between;
    margin-top: 9px;
    color: #8392a7;
    font-size: 11px;
}

.update-progress-meta strong {
    color: #a9caff;
    font-weight: 650;
}

.update-modal .modal-actions {
    justify-content: center;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

@keyframes wave {
    0%, 100% { transform: scaleY(0.6); opacity: 0.72; }
    50% { transform: scaleY(1); opacity: 1; }
}

/* System-aware console palette. Layout and component behavior stay unchanged. */
.fi-shell {
    color-scheme: dark;
    --fi-bg: #111315;
    --fi-chrome: #151719;
    --fi-sidebar: #141618;
    --fi-surface: #1b1e21;
    --fi-surface-raised: #22262a;
    --fi-surface-soft: #2b3036;
    --fi-border: rgba(255, 255, 255, 0.08);
    --fi-border-strong: rgba(255, 255, 255, 0.13);
    --fi-text: #f4f5f6;
    --fi-text-soft: #c9cdd2;
    --fi-muted: #9299a2;
    --fi-faint: #747c86;
    --fi-accent: #6b93d6;
    --fi-accent-strong: #4f7fc9;
    --fi-accent-soft: rgba(107, 147, 214, 0.15);
    --fi-accent-faint: rgba(107, 147, 214, 0.08);
    --fi-success: #4fc58a;
    --fi-warning: #d5a95f;
    --fi-danger: #d66b70;
    --fi-control: #353b42;
    --fi-thumb: #f4f5f6;
    --fi-glass: rgba(20, 22, 25, 0.84);
    --fi-preview: #171a1e;
    --fi-shadow: rgba(0, 0, 0, 0.28);
    background: var(--fi-bg);
    border-color: var(--fi-border-strong);
    color: var(--fi-text);
}

.fi-titlebar {
    background: var(--fi-chrome);
    border-bottom-color: var(--fi-border);
}

.sidebar {
    background: var(--fi-sidebar);
    border-right-color: var(--fi-border);
}

.title-brand,
.brand-block h1,
.page-head h2,
.group-head h3,
.setting-row strong,
.preview-head h3,
.track-copy strong,
.lyric-copy strong,
.notify-copy strong,
.lyric-stack strong,
.metric-card strong,
.about-card h3,
.modal-card h3 {
    color: var(--fi-text);
}

.time-pill,
.mock-monitor strong,
.round-play {
    color: var(--fi-text);
}

.title-status,
.page-head p,
.setting-row p,
.preview-head span,
.side-preview-card > span,
.theme-preview > span,
.live-dot,
.preview-note p,
.side-preview-card p,
.app-source p,
.lyric-stack p,
.metric-card span,
.metric-card p,
.about-card p,
.update-progress-meta {
    color: var(--fi-muted);
}

.track-copy small,
.lyric-copy small,
.notify-copy small,
.speed-pill,
.mock-monitor,
.taskbar-preview strong,
.range-control strong,
.check-item,
.secondary-btn,
.select {
    color: var(--fi-text-soft);
}

.brand-mark,
.logo-mark,
.about-logo {
    color: var(--fi-accent);
}

.brand-block p,
.group-head span,
.text-button,
.metric-icon,
.app-source > span,
.app-dot,
.preview-note,
.update-modal-icon,
.update-progress-meta strong {
    color: var(--fi-accent);
}

.title-status,
.head-toggle,
.update-link,
.check-item {
    background: var(--fi-surface-raised);
}

.window-actions button,
.nav-item,
.preview-tab,
.segmented button,
.ghost-btn {
    color: var(--fi-muted);
}

.window-actions button:hover,
.nav-item:hover,
.nav-item.is-active,
.update-link:hover {
    color: var(--fi-text);
    background: var(--fi-accent-soft);
}

.window-actions button:last-child:hover {
    background: var(--fi-danger);
}

.nav-item.is-active::before,
.progress-line i,
.mini-meter i,
.audio-wave i,
.metric-card path,
.update-progress i {
    background: var(--fi-accent);
}

.metric-card path {
    stroke: var(--fi-accent);
}

.nav-item em {
    color: var(--fi-warning);
    border-color: color-mix(in srgb, var(--fi-warning) 42%, transparent);
}

.title-status i,
.live-dot i,
.status-dot {
    background: var(--fi-success);
    box-shadow: none;
}

.live-dot i.is-muted {
    background: var(--fi-faint);
}

.setting-group,
.preview-panel,
.side-preview-card,
.theme-preview,
.metric-card,
.about-card {
    background: var(--fi-surface);
    border-color: var(--fi-border);
}

.setting-row,
.app-source {
    border-top-color: var(--fi-border);
}

.settings-pane::-webkit-scrollbar-thumb,
.single-page::-webkit-scrollbar-thumb {
    background: var(--fi-control);
}

.ui-switch,
.native-switch {
    background: var(--fi-control);
}

.ui-switch span,
.native-switch span {
    background: var(--fi-thumb);
}

.ui-switch:has(input:checked),
.native-switch:has(input:checked),
.segmented button.is-active,
.preview-tab.is-active,
.primary-btn {
    background: var(--fi-accent-strong);
    color: #ffffff;
}

.segmented,
.position-card,
.preview-tab,
.app-source > span,
.secondary-btn,
.select {
    background: var(--fi-surface-raised);
}

.swatch {
    box-shadow: inset 0 0 0 2px var(--fi-border-strong);
}

.swatch.is-active::after {
    border-color: var(--fi-accent);
}

.swatch.custom {
    color: var(--fi-text-soft);
    background: var(--fi-surface-raised);
}

.range-control input,
.check-item input {
    accent-color: var(--fi-accent-strong);
}

.position-card {
    color: var(--fi-muted);
}

.position-card i {
    border-color: var(--fi-border-strong);
}

.position-card i::before {
    background: var(--fi-accent);
}

.position-card.is-active {
    color: var(--fi-text);
    background: var(--fi-accent-soft);
}

.text-button,
.metric-icon,
.app-dot,
.update-modal-icon {
    background: var(--fi-accent-soft);
}

.danger-btn {
    background: var(--fi-danger);
}

.preview-panel {
    background: var(--fi-surface);
}

.desktop-frame {
    background: linear-gradient(160deg, var(--fi-preview), var(--fi-surface-raised));
}

.wallpaper-glow {
    background: radial-gradient(circle at 55% 34%, var(--fi-accent-soft), transparent 42%);
    filter: blur(12px);
}

.taskbar-preview,
.mock-island {
    background: var(--fi-glass);
    box-shadow: inset 0 0 0 1px var(--fi-border-strong), 0 12px 28px var(--fi-shadow);
}

.taskbar-preview span,
.progress-line,
.mini-meter,
.round-play {
    background: var(--fi-border-strong);
}

.cover-art,
.lyric-cover {
    background: linear-gradient(135deg, var(--fi-surface-soft), var(--fi-accent));
}

.panda-face {
    background: var(--fi-text-soft);
}

.panda-face::before,
.panda-face::after,
.panda-face i,
.panda-face b {
    background: var(--fi-surface-raised);
}

.preview-note,
.status-pill.ok {
    background: var(--fi-accent-faint);
}

.status-pill.ok {
    color: var(--fi-success);
}

.mini-window {
    background: var(--fi-surface-raised);
}

.mini-window i {
    background: var(--fi-accent-soft);
}

.mini-window div {
    background: var(--fi-border-strong);
}

.modal-overlay {
    background: rgba(0, 0, 0, 0.54);
}

.modal-card {
    background: var(--fi-surface);
    border-color: var(--fi-border-strong);
    box-shadow: 0 24px 70px var(--fi-shadow);
}

.modal-card p {
    color: var(--fi-muted);
}

.update-progress {
    background: var(--fi-surface-soft);
}

.update-progress i {
    box-shadow: none;
}

.fi-shell.is-light {
    color-scheme: light;
    --fi-bg: #f3f4f6;
    --fi-chrome: #ffffff;
    --fi-sidebar: #f8f9fb;
    --fi-surface: #ffffff;
    --fi-surface-raised: #f0f2f5;
    --fi-surface-soft: #e4e8ed;
    --fi-border: rgba(28, 32, 38, 0.10);
    --fi-border-strong: rgba(28, 32, 38, 0.16);
    --fi-text: #20242a;
    --fi-text-soft: #4e5661;
    --fi-muted: #6f7782;
    --fi-faint: #8b949e;
    --fi-accent: #4878bd;
    --fi-accent-strong: #3267ad;
    --fi-accent-soft: rgba(72, 120, 189, 0.13);
    --fi-accent-faint: rgba(72, 120, 189, 0.07);
    --fi-success: #278a59;
    --fi-warning: #9b6a1d;
    --fi-danger: #c25359;
    --fi-control: #c9d0d8;
    --fi-thumb: #ffffff;
    --fi-glass: rgba(255, 255, 255, 0.84);
    --fi-preview: #dfe5ec;
    --fi-shadow: rgba(31, 41, 55, 0.14);
}

.fi-shell.is-light .modal-overlay {
    background: rgba(32, 36, 42, 0.28);
}

@media (max-width: 920px) {
    .fi-body {
        grid-template-columns: 178px minmax(0, 1fr);
    }

    .page-layout.has-preview,
    .two-column-page {
        grid-template-columns: 1fr;
    }

    .category-grid {
        grid-template-columns: 1fr;
    }

    .category-grid .setting-group:last-child {
        grid-column: auto;
    }

    .preview-panel {
        min-height: 430px;
    }
}
</style>
