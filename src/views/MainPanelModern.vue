<template>
    <div class="modern-shell">
        <div class="window-bar" data-tauri-drag-region>
            <div class="window-title" data-tauri-drag-region>FlowIsland 控制台</div>
            <div class="window-actions">
                <button class="window-button" aria-label="最小化" @click="minimizeWindow">
                    <Minus :size="16" />
                </button>
                <button class="window-button close" aria-label="关闭" @click="closeWindow">
                    <X :size="17" />
                </button>
            </div>
        </div>

        <aside class="sidebar">
            <div class="brand-panel">
                <div class="brand-logo">
                    <img src="../assets/logo.png" alt="FlowIsland">
                </div>
                <h1>FlowIsland</h1>
                <p>桌面灵动岛组件</p>
                <span>v{{ appVersion }}</span>
            </div>

            <nav class="side-nav">
                <button v-for="item in navItems" :key="item.id" class="nav-item"
                    :class="{ 'is-active': activeSection === item.id }" @click="activeSection = item.id">
                    <component :is="item.icon" :size="22" />
                    <span>{{ item.label }}</span>
                </button>
            </nav>

            <div class="sidebar-footer">
                <button class="update-button" :disabled="isChecking" @click="checkUpdate">
                    <CloudDownload :size="20" />
                    <span>{{ isChecking ? '检查中' : (hasNewVersion ? '发现新版本' : '检查更新') }}</span>
                    <i v-if="hasNewVersion"></i>
                </button>
                <p>&copy; 2026 Ryen. All rights reserved.</p>
            </div>
        </aside>

        <main class="content">
            <section class="hero-card">
                <div class="hero-icon">
                    <AudioLines :size="34" />
                </div>
                <div class="hero-copy">
                    <h2>FlowIsland</h2>
                    <p>桌面灵动岛组件 v{{ appVersion }}</p>
                    <span>灵动岛组件已在后台运行，实时展示系统状态与信息。</span>
                </div>
                <div class="hero-switch" :class="{ 'is-active': isWidgetVisible }">
                    <strong>{{ isWidgetVisible ? '已开启' : '已关闭' }}</strong>
                    <label class="switch">
                        <input type="checkbox" :checked="isWidgetVisible" @change="toggleWidget">
                        <span class="slider"></span>
                    </label>
                </div>
            </section>

            <div class="content-scroll">
                <section v-show="activeSection === 'island'" class="workspace">
                    <div class="settings-column">
                        <SettingGroup title="外观与识别" :icon="Palette">
                            <SettingRow :icon="Palette" title="灵动岛颜色" desc="切换灵动岛的默认背景色调">
                                <div class="segment-control">
                                    <button :class="{ 'is-active': islandTheme === 'system' }"
                                        @click="islandTheme = 'system'">跟随</button>
                                    <button :class="{ 'is-active': islandTheme === 'black' }"
                                        @click="islandTheme = 'black'">暗色</button>
                                    <button :class="{ 'is-active': islandTheme === 'white' }"
                                        @click="islandTheme = 'white'">亮色</button>
                                </div>
                            </SettingRow>
                            <SettingRow :icon="Music2" title="音乐识别" desc="检测到主流音乐软件时自动显示音乐灵动岛">
                                <ToggleSwitch v-model="enableMusicCtrl" />
                            </SettingRow>
                        </SettingGroup>

                        <SettingGroup title="消息与任务" :icon="MessagesSquare">
                            <SettingRow :icon="Bell" title="消息通知接收" desc="启用系统控制中心消息弹窗提醒">
                                <ToggleSwitch v-model="enableMsgNotify" @change="toggleMsgNotify" />
                            </SettingRow>
                            <SettingRow :icon="PanelBottom" title="置于任务栏" desc="将灵动岛锁定至任务栏左下角" badge="♛">
                                <ToggleSwitch v-model="pinToTaskbar" @change="togglePinTaskbar" />
                            </SettingRow>
                        </SettingGroup>

                        <SettingGroup title="行为模式" :icon="Zap">
                            <SettingRow :icon="MessageCircle" title="消息模式" desc="平时自动隐藏，收到消息后才弹出">
                                <ToggleSwitch v-model="msgModeEnabled" @change="toggleMsgMode" />
                            </SettingRow>
                            <SettingRow :icon="Gauge" title="灵动岛不透明度" :desc="`当前 ${opacity}%`">
                                <input v-model="opacity" type="range" min="0" max="100" class="range-input">
                            </SettingRow>
                        </SettingGroup>
                    </div>

                    <PreviewPanel :running="isWidgetVisible" :opacity="opacity" />
                </section>

                <section v-show="activeSection === 'notify'" class="workspace single">
                    <SettingGroup title="通知与消息" :icon="Bell">
                        <SettingRow :icon="Bell" title="消息通知接收" desc="接收 Windows 通知并以消息岛方式弹出">
                            <ToggleSwitch v-model="enableMsgNotify" @change="toggleMsgNotify" />
                        </SettingRow>
                        <SettingRow :icon="MessageCircle" title="消息模式" desc="平时隐藏，收到消息后以弹出方式提醒">
                            <ToggleSwitch v-model="msgModeEnabled" @change="toggleMsgMode" />
                        </SettingRow>
                    </SettingGroup>
                    <InfoCard title="提示" text="消息岛只在收到通知时出现，平时不会遮挡桌面内容。" />
                </section>

                <section v-show="activeSection === 'system'" class="workspace single">
                    <SettingGroup title="系统与显示" :icon="Monitor">
                        <SettingRow :icon="Monitor" title="控制台主题" desc="软件设置界面默认使用深色蓝光风格">
                            <select v-model="themeMode" class="select-input" @change="handleThemeChange">
                                <option value="system">跟随系统</option>
                                <option value="dark">深色模式</option>
                                <option value="light">浅色模式</option>
                            </select>
                        </SettingRow>
                        <SettingRow :icon="Rocket" title="开机自启动" desc="登录 Windows 后自动启动 FlowIsland">
                            <ToggleSwitch v-model="autoStart" @change="toggleAutoStart" />
                        </SettingRow>
                        <SettingRow :icon="Gauge" title="歌词同步偏移" :desc="`当前 ${lyricOffsetMs}ms`">
                            <input v-model.number="lyricOffsetMs" type="range" min="-3000" max="3000" step="100"
                                class="range-input">
                        </SettingRow>
                    </SettingGroup>
                    <InfoCard title="运行状态" :text="`上传 ${uploadSpeed} · 下载 ${downloadSpeed}`" />
                </section>

                <section v-show="activeSection === 'about'" class="workspace single">
                    <SettingGroup title="关于与更新" :icon="Info">
                        <SettingRow :icon="Info" title="当前版本" desc="FlowIsland">
                            <strong class="value-pill">v{{ appVersion }}</strong>
                        </SettingRow>
                        <SettingRow :icon="CloudDownload" title="检查更新" desc="从发布源检查新版安装包">
                            <button class="primary-button" :disabled="isChecking" @click="checkUpdate">
                                {{ isChecking ? '检查中' : '立即检查' }}
                            </button>
                        </SettingRow>
                    </SettingGroup>
                    <InfoCard title="项目" text="由 Ryen 开发维护，专注于桌面灵动岛体验。" />
                </section>
            </div>
        </main>

        <Transition name="fade">
            <div v-if="dialog.visible" class="modal-overlay" @click.self="closeDialog">
                <div class="modal-card">
                    <h3>{{ dialog.title }}</h3>
                    <p>{{ dialog.message }}</p>
                    <div class="modal-actions">
                        <button v-if="dialog.isConfirm" class="secondary-button" @click="closeDialog">取消</button>
                        <button class="primary-button" @click="handleDialogConfirm">确定</button>
                    </div>
                </div>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, nextTick, onMounted, onUnmounted, PropType, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import {
    AudioLines,
    Bell,
    CloudDownload,
    Gauge,
    Hexagon,
    Info,
    MessageCircle,
    MessagesSquare,
    Minus,
    Monitor,
    Music2,
    Palette,
    PanelBottom,
    Rocket,
    Sparkles,
    X,
    Zap,
} from 'lucide-vue-next';

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

type NavId = 'island' | 'notify' | 'system' | 'about';

const activeSection = ref<NavId>('island');
const isWidgetVisible = ref(false);
const autoStart = ref(false);
const appVersion = ref('2.3.5');
const uploadSpeed = ref('0 B/s');
const downloadSpeed = ref('0 B/s');
const opacity = ref(Number(localStorage.getItem('nsd_island_opacity') || '30'));
const lyricOffsetMs = ref(Number(localStorage.getItem('nsd_lyric_offset_ms') || '0'));
const themeMode = ref(localStorage.getItem('nsd_theme_mode') || 'system');
const islandTheme = ref(localStorage.getItem('nsd_island_theme') || 'system');
const enableMusicCtrl = ref(localStorage.getItem('nsd_music_ctrl') !== 'false');
const enableMsgNotify = ref(localStorage.getItem('nsd_msg_notify') !== 'false');
const msgModeEnabled = ref(localStorage.getItem('nsd_msg_mode') !== 'false');
const pinToTaskbar = ref(localStorage.getItem('nsd_pin_taskbar') === 'true');
const isChecking = ref(false);
const hasNewVersion = ref(false);

const navItems = [
    { id: 'island' as const, label: '灵动岛设置', icon: Hexagon },
    { id: 'notify' as const, label: '通知与消息', icon: Bell },
    { id: 'system' as const, label: '系统与显示', icon: Monitor },
    { id: 'about' as const, label: '关于与更新', icon: Info },
];

const dialog = ref({
    visible: false,
    title: '',
    message: '',
    isConfirm: false,
    onConfirm: null as (() => void) | null,
});

let speedTimer: number | null = null;
let lastRx = 0;
let lastTx = 0;

const SettingGroup = defineComponent({
    props: {
        title: { type: String, required: true },
        icon: { type: Object as PropType<any>, required: true },
    },
    setup(props, { slots }) {
        return () => h('section', { class: 'setting-group' }, [
            h('div', { class: 'group-title' }, [
                h(props.icon, { size: 22 }),
                h('span', props.title),
            ]),
            h('div', { class: 'group-card' }, slots.default?.()),
        ]);
    },
});

const SettingRow = defineComponent({
    props: {
        title: { type: String, required: true },
        desc: { type: String, required: true },
        icon: { type: Object as PropType<any>, required: true },
        badge: { type: String, default: '' },
    },
    setup(props, { slots }) {
        return () => h('div', { class: 'setting-row' }, [
            h('div', { class: 'row-icon' }, [h(props.icon, { size: 23 })]),
            h('div', { class: 'row-copy' }, [
                h('strong', [
                    props.title,
                    props.badge ? h('small', { class: 'gold-badge' }, props.badge) : null,
                ]),
                h('span', props.desc),
            ]),
            h('div', { class: 'row-control' }, slots.default?.()),
        ]);
    },
});

const ToggleSwitch = defineComponent({
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
        return () => h('label', { class: 'switch' }, [
            h('input', { type: 'checkbox', checked: props.modelValue, onChange }),
            h('span', { class: 'slider' }),
        ]);
    },
});

const PreviewPanel = defineComponent({
    props: {
        running: { type: Boolean, required: true },
        opacity: { type: Number, required: true },
    },
    setup(props) {
        const islandStyle = computed(() => ({
            background: `rgba(9, 17, 35, ${Math.max(0.28, props.opacity / 100)})`,
        }));

        return () => h('aside', { class: 'preview-panel' }, [
            h('div', { class: 'preview-heading' }, [
                h('div', [h('h3', '灵动岛预览'), h('p', '当前音乐岛与消息岛的展示样式')]),
                h('span', { class: ['running-dot', props.running ? 'is-active' : ''] }, props.running ? '运行中' : '已关闭'),
            ]),
            h('div', { class: 'preview-screen' }, [
                h('div', { class: 'preview-orbit' }),
                h('div', { class: 'preview-island', style: islandStyle.value }, [
                    h('div', { class: 'preview-cover' }, [h(Music2, { size: 18 })]),
                    h('div', { class: 'preview-track' }, [
                        h('strong', 'Dream It Possible'),
                        h('span', '音乐播放时自动显示'),
                    ]),
                    h('div', { class: 'mini-wave' }, [1, 2, 3, 4].map((n) => h('i', { class: `bar-${n}` }))),
                ]),
                h('div', { class: 'preview-status-grid' }, [
                    h('div', [h(AudioLines, { size: 16 }), h('span', '音乐识别')]),
                    h('div', [h(Bell, { size: 16 }), h('span', '消息提醒')]),
                ]),
            ]),
            h('div', { class: 'tip-card' }, [
                h(Sparkles, { size: 22 }),
                h('div', [h('strong', '提示'), h('p', '音乐播放时显示音乐岛，消息到达时弹出消息岛。')]),
            ]),
        ]);
    },
});

const InfoCard = defineComponent({
    props: {
        title: { type: String, required: true },
        text: { type: String, required: true },
    },
    setup(props) {
        return () => h('aside', { class: 'preview-panel compact-panel' }, [
            h('div', { class: 'tip-card only' }, [
                h(Sparkles, { size: 22 }),
                h('div', [h('strong', props.title), h('p', props.text)]),
            ]),
        ]);
    },
});

const formatSpeed = (bytes: number) => {
    if (bytes < 1024) return `${bytes.toFixed(0)} B/s`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB/s`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
};

const fetchSpeedStats = async () => {
    try {
        const [currentRx, currentTx] = await invoke<[number, number]>('get_network_stats');
        if (lastRx !== 0 && lastTx !== 0) {
            downloadSpeed.value = formatSpeed(Math.max(0, (currentRx - lastRx) / 2));
            uploadSpeed.value = formatSpeed(Math.max(0, (currentTx - lastTx) / 2));
        }
        lastRx = currentRx;
        lastTx = currentTx;
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

const handleThemeChange = () => {
    localStorage.setItem('nsd_theme_mode', themeMode.value);
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

const checkUpdate = async () => {
    if (isChecking.value) return;
    isChecking.value = true;
    try {
        const response = await fetch('https://api.github.com/repos/GEORGEWWWU/NetSpeed-Dynamic/releases/latest', {
            headers: { Accept: 'application/vnd.github+json' },
        });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        const local = parseVersion(appVersion.value);
        const remote = parseVersion(data.tag_name || data.name || '');
        const hasUpdate = remote.some((num, index) => num > (local[index] || 0));
        hasNewVersion.value = hasUpdate;
        showDialog(hasUpdate ? '发现新版本' : '已是最新版本', hasUpdate ? `检测到新版本 ${data.tag_name}` : '当前已经是最新版本。');
    } catch (error) {
        showDialog('检查失败', '无法检查更新，请稍后再试。');
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
    localStorage.setItem('nsd_island_theme', value);
    await emit('control-island-theme', { theme: value });
});

watch(enableMusicCtrl, async (value) => {
    localStorage.setItem('nsd_music_ctrl', String(value));
    await emit('control-music-ctl', { enabled: value });
});

onMounted(async () => {
    try {
        appVersion.value = await getVersion();
    } catch { /* ignore */ }

    try {
        autoStart.value = await isEnabled();
    } catch { /* ignore */ }

    await fetchSpeedStats();
    speedTimer = window.setInterval(fetchSpeedStats, 2000);

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
});
</script>

<style>
html,
body,
#app {
    width: 100%;
    height: 100%;
    margin: 0;
    overflow: hidden;
    background: transparent;
}

body {
    font-family: Inter, "SF Pro Display", "Microsoft YaHei UI", "Segoe UI", sans-serif;
    color: #f7fbff;
    user-select: none;
    -webkit-font-smoothing: antialiased;
}

.modern-shell {
    width: 100%;
    height: 100vh;
    display: grid;
    grid-template-columns: 190px minmax(0, 1fr);
    grid-template-rows: 34px minmax(0, 1fr);
    background:
        radial-gradient(circle at 18% 9%, rgba(42, 112, 255, 0.22), transparent 18%),
        radial-gradient(circle at 82% 0%, rgba(62, 137, 255, 0.14), transparent 26%),
        linear-gradient(135deg, #020711 0%, #050a15 48%, #07101e 100%);
    border: 1px solid rgba(112, 148, 213, 0.26);
    box-sizing: border-box;
    overflow: hidden;
    color: #edf5ff;
    font-size: 12px;
}

.window-bar {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px 0 18px;
    background: rgba(3, 8, 18, 0.72);
    border-bottom: 1px solid rgba(90, 124, 178, 0.14);
}

.window-title {
    color: rgba(203, 213, 225, 0.72);
    font-size: 12px;
    letter-spacing: 0.2px;
}

.window-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.window-button {
    width: 30px;
    height: 24px;
    border: 0;
    border-radius: 8px;
    color: rgba(226, 232, 240, 0.86);
    background: transparent;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
}

.window-button:hover {
    background: rgba(71, 106, 170, 0.22);
}

.window-button.close:hover {
    color: #fff;
    background: rgba(239, 68, 68, 0.82);
}

.sidebar {
    min-height: 0;
    padding: 18px 10px 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    border-right: 1px solid rgba(102, 139, 211, 0.18);
    background:
        linear-gradient(180deg, rgba(18, 33, 58, 0.76), rgba(4, 10, 21, 0.84)),
        rgba(6, 13, 28, 0.82);
    box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.03);
}

.brand-panel {
    width: 100%;
    text-align: center;
    padding: 12px 8px 12px;
}

.brand-logo {
    width: 74px;
    height: 74px;
    margin: 0 auto 12px;
    border-radius: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(150deg, rgba(36, 55, 85, 0.9), rgba(14, 33, 70, 0.86));
    border: 1px solid rgba(95, 139, 225, 0.42);
    box-shadow: 0 0 34px rgba(37, 99, 235, 0.55), inset 0 0 22px rgba(91, 141, 255, 0.24);
}

.brand-logo img {
    width: 54px;
    height: 54px;
    object-fit: contain;
}

.brand-panel h1 {
    margin: 0;
    color: #f8fbff;
    font-size: 17px;
    line-height: 1.12;
    font-weight: 800;
    letter-spacing: -0.3px;
}

.brand-panel p {
    margin: 9px 0 3px;
    color: rgba(176, 188, 211, 0.78);
    font-size: 11px;
}

.brand-panel span {
    color: #3d86ff;
    font-weight: 700;
}

.side-nav {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 10px;
}

.nav-item {
    height: 46px;
    padding: 0 15px;
    border: 0;
    border-radius: 12px;
    color: rgba(190, 201, 222, 0.76);
    background: transparent;
    display: flex;
    align-items: center;
    gap: 11px;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
}

.nav-item svg {
    flex: none;
}

.nav-item span {
    flex: 1;
    text-align: left;
}

.nav-item small {
    border: 1px solid rgba(245, 184, 82, 0.48);
    color: #ffd484;
    border-radius: 6px;
    padding: 1px 6px;
    font-size: 11px;
    line-height: 18px;
}

.nav-item:hover {
    background: rgba(35, 64, 112, 0.34);
    color: #f8fbff;
}

.nav-item.is-active {
    color: #fff;
    background: linear-gradient(135deg, #1e6cff, #0b3272);
    box-shadow: 0 16px 30px rgba(0, 82, 255, 0.32), inset 0 1px 0 rgba(255, 255, 255, 0.14);
}

.sidebar-footer {
    width: 100%;
    margin-top: auto;
    padding: 0 16px;
    box-sizing: border-box;
}

.update-button {
    width: 100%;
    height: 36px;
    border: 1px solid rgba(110, 143, 199, 0.24);
    border-radius: 15px;
    color: rgba(226, 232, 240, 0.9);
    background: rgba(18, 29, 49, 0.72);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
}

.update-button:hover {
    border-color: rgba(68, 132, 255, 0.62);
    box-shadow: 0 0 22px rgba(37, 99, 235, 0.18);
}

.update-button i {
    width: 7px;
    height: 7px;
    border-radius: 99px;
    background: #22c55e;
}

.sidebar-footer p {
    margin: 18px 0 0;
    color: rgba(148, 163, 184, 0.72);
    font-size: 12px;
    text-align: center;
}

.content {
    min-width: 0;
    min-height: 0;
    padding: 14px 18px 18px 20px;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 14px;
}

.hero-card,
.preview-panel,
.group-card {
    border: 1px solid rgba(93, 130, 198, 0.22);
    background:
        linear-gradient(145deg, rgba(17, 32, 58, 0.78), rgba(7, 14, 28, 0.84)),
        rgba(8, 18, 35, 0.82);
    box-shadow: 0 20px 45px rgba(0, 0, 0, 0.28), inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.hero-card {
    min-height: 94px;
    padding: 0 22px;
    border-radius: 18px;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 14px;
}

.hero-icon {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    color: #8cc8ff;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle, rgba(37, 99, 235, 0.38), rgba(3, 10, 25, 0.92) 70%);
    border: 1px solid rgba(61, 133, 255, 0.62);
    box-shadow: 0 0 34px rgba(37, 99, 235, 0.56);
}

.hero-copy h2 {
    margin: 0;
    color: #f8fbff;
    font-size: 19px;
    letter-spacing: -0.4px;
}

.hero-copy p,
.hero-copy span {
    display: block;
    margin-top: 5px;
    color: rgba(188, 201, 222, 0.72);
    font-size: 12px;
}

.hero-switch {
    min-width: 124px;
    height: 40px;
    padding: 0 10px 0 16px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    color: #9ab2d9;
    background: rgba(24, 39, 68, 0.86);
    border: 1px solid rgba(93, 130, 198, 0.26);
}

.hero-switch.is-active {
    color: #5aa0ff;
    background: linear-gradient(135deg, rgba(32, 69, 132, 0.92), rgba(16, 36, 76, 0.92));
    border-color: rgba(59, 130, 246, 0.68);
}

.hero-switch strong {
    font-size: 13px;
}

.content-scroll {
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 8px;
    scrollbar-gutter: stable;
}

.content-scroll::-webkit-scrollbar {
    width: 8px;
}

.content-scroll::-webkit-scrollbar-track {
    background: transparent;
}

.content-scroll::-webkit-scrollbar-thumb {
    border: 2px solid transparent;
    border-radius: 99px;
    background: rgba(92, 128, 192, 0.36);
    background-clip: padding-box;
}

.workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    gap: 16px;
    align-items: start;
}

.workspace.single {
    grid-template-columns: minmax(0, 1fr) 300px;
}

.settings-column {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
}

.setting-group {
    display: flex;
    flex-direction: column;
    gap: 9px;
}

.group-title {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #2f7cff;
    font-size: 14px;
    font-weight: 800;
}

.group-title svg {
    color: #8ea4c9;
}

.group-card {
    border-radius: 15px;
    overflow: hidden;
}

.setting-row {
    min-height: 62px;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    padding: 0 16px 0 14px;
    border-bottom: 1px solid rgba(109, 137, 183, 0.14);
}

.setting-row:last-child {
    border-bottom: 0;
}

.row-icon {
    width: 36px;
    height: 36px;
    border-radius: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #f8fbff;
    background: linear-gradient(145deg, rgba(39, 54, 78, 0.9), rgba(20, 32, 52, 0.86));
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

.row-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.row-copy strong {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #f8fbff;
    font-size: 14px;
    line-height: 1.1;
}

.row-copy span {
    color: rgba(174, 187, 211, 0.78);
    font-size: 12px;
    line-height: 1.35;
}

.row-control {
    display: flex;
    align-items: center;
    justify-content: flex-end;
}

.gold-badge {
    color: #ffd76a;
}

.segment-control {
    height: 42px;
    padding: 4px;
    display: flex;
    border-radius: 999px;
    background: rgba(18, 29, 49, 0.92);
    border: 1px solid rgba(109, 137, 183, 0.22);
}

.segment-control button {
    min-width: 64px;
    border: 0;
    border-radius: 999px;
    color: rgba(205, 215, 232, 0.78);
    background: transparent;
    font-size: 12px;
    font-weight: 800;
    cursor: pointer;
}

.segment-control button.is-active {
    color: #fff;
    background: linear-gradient(135deg, #4a8fff, #1f63ee);
    box-shadow: 0 10px 22px rgba(37, 99, 235, 0.32);
}

.switch {
    position: relative;
    width: 50px;
    height: 28px;
    display: inline-block;
}

.switch input {
    width: 0;
    height: 0;
    opacity: 0;
}

.slider {
    position: absolute;
    inset: 0;
    border-radius: 999px;
    background: rgba(71, 85, 105, 0.82);
    cursor: pointer;
    transition: 0.22s ease;
}

.slider::before {
    content: "";
    position: absolute;
    width: 20px;
    height: 20px;
    top: 4px;
    left: 4px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 8px 18px rgba(0, 0, 0, 0.25);
    transition: 0.22s ease;
}

.switch input:checked + .slider {
    background: linear-gradient(135deg, #4a8fff, #1d5fff);
}

.switch input:checked + .slider::before {
    transform: translateX(22px);
}

.range-input {
    width: 140px;
    accent-color: #2f7cff;
}

.select-input,
.primary-button,
.secondary-button {
    height: 32px;
    border-radius: 12px;
    border: 1px solid rgba(109, 137, 183, 0.24);
    color: #f8fbff;
    background: rgba(18, 29, 49, 0.92);
    font-size: 12px;
    font-weight: 800;
}

.select-input {
    width: 150px;
    padding: 0 12px;
}

.primary-button,
.secondary-button {
    padding: 0 18px;
    cursor: pointer;
}

.primary-button {
    background: linear-gradient(135deg, #438cff, #1659e8);
    border-color: rgba(99, 155, 255, 0.68);
}

.secondary-button {
    background: rgba(31, 41, 55, 0.86);
}

.value-pill {
    min-width: 76px;
    padding: 9px 14px;
    border-radius: 12px;
    text-align: center;
    background: rgba(33, 51, 82, 0.8);
    color: #7db1ff;
}

.preview-panel {
    position: sticky;
    top: 0;
    border-radius: 18px;
    padding: 16px;
    min-height: 420px;
}

.compact-panel {
    min-height: auto;
}

.preview-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
}

.preview-heading h3 {
    margin: 0;
    color: #f8fbff;
    font-size: 17px;
}

.preview-heading p {
    margin: 10px 0 0;
    color: rgba(182, 195, 217, 0.75);
    line-height: 1.5;
}

.running-dot {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: rgba(182, 195, 217, 0.7);
    white-space: nowrap;
}

.running-dot::before {
    content: "";
    width: 10px;
    height: 10px;
    border-radius: 99px;
    background: #64748b;
}

.running-dot.is-active::before {
    background: #22c55e;
    box-shadow: 0 0 18px rgba(34, 197, 94, 0.75);
}

.preview-screen {
    position: relative;
    height: 236px;
    margin-top: 12px;
    border-radius: 15px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 26px;
    background:
        radial-gradient(circle at 50% 36%, rgba(64, 136, 255, 0.26), transparent 35%),
        linear-gradient(180deg, #0f2a69, #020914 78%);
    border: 1px solid rgba(95, 139, 225, 0.18);
}

.preview-orbit {
    position: absolute;
    width: 190px;
    height: 190px;
    border: 2px solid rgba(95, 165, 255, 0.72);
    border-radius: 50%;
    box-shadow: 0 0 34px rgba(59, 130, 246, 0.32);
    opacity: 0.9;
}

.preview-orbit::before,
.preview-orbit::after {
    content: "";
    position: absolute;
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.82);
    box-shadow: 42px 42px 0 rgba(255, 255, 255, 0.62), 142px 30px 0 rgba(255, 255, 255, 0.72);
}

.preview-orbit::before {
    left: 24px;
    top: 28px;
}

.preview-island {
    position: relative;
    z-index: 1;
    width: 228px;
    height: 48px;
    border-radius: 999px;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    padding: 0 14px 0 8px;
    border: 1px solid rgba(107, 168, 255, 0.42);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.34), 0 0 24px rgba(59, 130, 246, 0.24);
    backdrop-filter: blur(16px);
}

.preview-cover {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #dbeafe;
    background: rgba(92, 121, 174, 0.38);
}

.preview-track {
    min-width: 0;
}

.preview-track strong {
    display: block;
    overflow: hidden;
    color: #fff;
    font-size: 12px;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.preview-track span {
    display: block;
    margin-top: 2px;
    color: rgba(203, 213, 225, 0.7);
    font-size: 10px;
}

.mini-wave {
    height: 24px;
    display: flex;
    align-items: center;
    gap: 3px;
}

.mini-wave i {
    width: 3px;
    border-radius: 99px;
    background: #0ea5e9;
    animation: wave 0.75s ease-in-out infinite;
}

.mini-wave .bar-1 { height: 8px; }
.mini-wave .bar-2 { height: 16px; animation-delay: 0.12s; }
.mini-wave .bar-3 { height: 22px; animation-delay: 0.22s; }
.mini-wave .bar-4 { height: 12px; animation-delay: 0.32s; }

.preview-status-grid {
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    width: 228px;
}

.preview-status-grid div {
    height: 38px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    color: rgba(219, 234, 254, 0.86);
    font-size: 12px;
    background: rgba(9, 20, 42, 0.58);
    border: 1px solid rgba(107, 168, 255, 0.22);
}

.tip-card {
    margin-top: 14px;
    padding: 15px;
    display: flex;
    gap: 10px;
    border-radius: 18px;
    border: 1px solid rgba(93, 130, 198, 0.18);
    background: rgba(13, 24, 43, 0.62);
}

.tip-card.only {
    margin-top: 0;
}

.tip-card svg {
    color: #4f91ff;
    flex: none;
}

.tip-card strong {
    color: #f8fbff;
    font-size: 13px;
}

.tip-card p {
    margin: 8px 0 0;
    color: rgba(182, 195, 217, 0.76);
    font-size: 12px;
    line-height: 1.55;
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
}

.modal-actions {
    margin-top: 22px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
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

@media (max-width: 820px) {
    .modern-shell {
        grid-template-columns: 170px minmax(0, 1fr);
    }

    .workspace,
    .workspace.single {
        grid-template-columns: 1fr;
    }

    .preview-panel {
        position: static;
        min-height: auto;
    }

    .hero-card {
        grid-template-columns: auto minmax(0, 1fr);
    }

    .hero-switch {
        grid-column: 1 / -1;
        justify-self: start;
    }
}
</style>
