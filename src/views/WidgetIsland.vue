<template>
    <transition @enter="onEnter" @leave="onLeave" :css="false">
        <div v-show="isIslandVisible"
            :class="['island-container', { 'has-music-border': isGlowBorderEnabled, 'is-split-mode': isSplitActivity, 'is-default-dashboard': isDefaultSpeedPrimary, 'is-hardware-dashboard': isHardwarePrimary }]"
            @mousedown="handleMouseDown" :style="islandStyle" @contextmenu="handleRightClick">

            <div class="rainbow-border-glow" v-if="showFlowBorder" :style="albumBorderStyle">
            </div>

            <div v-if="isSplitActivity" class="split-music-bubble" :style="splitMusicBubbleStyle" title="打开音乐播放器"
                @mousedown.stop @click.stop="openMusicApp">
                <div :class="['music-mini-cover', { 'is-playing': isPlaying }]"
                    :style="coverUrl ? { backgroundImage: `url(${coverUrl})`, backgroundSize: 'cover' } : {}">
                </div>
                <button class="music-mini-play" :title="isPlaying ? '暂停' : '播放'" @mousedown.stop
                    @click.stop="togglePlay">
                    <svg v-if="isPlaying" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                    </svg>
                    <svg v-else viewBox="0 0 24 24" fill="currentColor" style="transform: translateX(1px);">
                        <path d="M8 5v14l11-7z" />
                    </svg>
                </button>
            </div>

            <div class="island-core-content" :style="splitCoreStyle">
                <div class="inner-wrapper">
                    <transition @enter="onInnerEnter" @leave="onInnerLeave" :css="false">
                        <div class="msg-box" v-show="isMsgActive" key="msg" @click="handleMsgClick"
                            style="cursor: pointer;">
                            <div class="msg-avatar">
                                <img :src="currentMsgIcon" alt="消息图标" class="msg-avatar-img">
                            </div>

                            <div class="msg-text-wrapper">
                                <div class="msg-title">{{ msgTitle }}</div>
                                <div class="msg-body">{{ msgBody }}</div>
                            </div>
                        </div>
                    </transition>

                    <transition @enter="onInnerEnter" @leave="onInnerLeave" :css="false">
                        <div class="speed-box default-stats-box hardware-stats-box"
                            v-show="isHardwarePrimary && !isMsgActive" key="hardware" :style="hardwareGridStyle">
                            <div v-if="showCpuMetric" class="default-stat-item default-stat-compute">
                                <div class="default-stat-icon">
                                    <Cpu class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">CPU</div>
                                    <div class="default-stat-values">
                                        <span>{{ cpuUsage }}</span>
                                        <span v-if="hasCpuTemp">{{ cpuTemp }}</span>
                                        <span v-if="hasCpuFan">{{ cpuFan }}</span>
                                    </div>
                                </div>
                            </div>
                            <div v-if="showGpuMetric" class="default-stat-item default-stat-compute">
                                <div class="default-stat-icon">
                                    <CircuitBoard class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">显卡</div>
                                    <div class="default-stat-values">
                                        <span>{{ gpuUsage }}</span>
                                        <span>{{ gpuTemp }}</span>
                                        <span>{{ gpuFan }}</span>
                                    </div>
                                </div>
                            </div>
                            <div v-if="showGpuMetric" class="default-stat-item default-stat-memory">
                                <div class="default-stat-icon">
                                    <Database class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">显存</div>
                                    <div class="default-stat-values">
                                        <span>{{ gpuMemoryUsage }}</span>
                                        <span>{{ gpuMemory }}</span>
                                    </div>
                                </div>
                            </div>
                            <div v-if="showMemoryMetric" class="default-stat-item default-stat-memory">
                                <div class="default-stat-icon">
                                    <MemoryStick class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">内存</div>
                                    <div class="default-stat-values">
                                        <span>{{ ramUsage }}</span>
                                        <span>{{ ramMemory }}</span>
                                    </div>
                                </div>
                            </div>
                            <div v-if="showNetworkMetric" class="default-stat-item default-stat-network">
                                <div class="default-stat-icon">
                                    <Wifi class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">网络</div>
                                    <div class="default-stat-values network-values">
                                        <span :class="{ 'high-traffic': isHighDownload }">{{ downloadSpeed }}↓</span>
                                        <span :class="{ 'high-traffic': isHighUpload }">{{ uploadSpeed }}↑</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </transition>

                    <transition @enter="onInnerEnter" @leave="onInnerLeave" :css="false">
                        <div class="music-ctl-box" v-show="isMusicPrimary && !isMsgActive" :key="musicBoxKey"
                            @mouseenter="handleMusicBoxEnter" @mouseleave="handleMusicBoxLeave">

                            <div class="album-cover" :class="{ 'is-playing': isPlaying }" title="打开音乐播放器"
                                @mousedown.stop @click.stop="openMusicApp">
                                <div class="cover-inner"
                                    :style="coverUrl ? { backgroundImage: `url(${coverUrl})`, backgroundSize: 'cover' } : {}">
                                </div>
                            </div>

                            <transition name="fade">
                                <div class="music-controls" v-show="!showInfo">
                                    <button class="ctl-btn" @click="prevTrack">
                                        <svg viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" />
                                        </svg>
                                    </button>

                                    <button class="ctl-btn play-btn" @click="togglePlay">
                                        <svg v-if="isPlaying" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                                        </svg>
                                        <svg v-else viewBox="0 0 24 24" fill="currentColor"
                                            style="transform: translateX(1px);">
                                            <path d="M8 5v14l11-7z" />
                                        </svg>
                                    </button>

                                    <button class="ctl-btn" @click="nextTrack">
                                        <svg viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" />
                                        </svg>
                                    </button>
                                </div>
                            </transition>
                        </div>
                    </transition>

                    <transition name="fade">
                        <div class="music-info-mask-box" v-show="isMusicPrimary && !isMsgActive && showInfo">
                            <div :class="['music-info-text', { 'is-marquee': isLongTrackTitle, 'has-lyric': hasDisplayLyricLine }]">
                                <div ref="musicTitleViewportRef" class="music-title-viewport">
                                    <div class="music-title-track">
                                        <span ref="musicTitleTextRef" class="music-title">{{ currentTrackInfo }}</span>
                                        <span v-if="isLongTrackTitle" class="music-title music-title-duplicate">
                                            {{ currentTrackInfo }}
                                        </span>
                                    </div>
                                </div>
                                <transition name="lyric-fade" mode="out-in">
                                    <div v-if="hasDisplayLyricLine" :key="displayLyricLine"
                                        :class="['music-lyric-line', { 'is-long': isLongDisplayLyricLine, 'is-upcoming': isUpcomingLyricLine }]"
                                        :title="lyricsSource ? `歌词来源：${lyricsSource}` : ''">
                                        <div ref="musicLyricViewportRef" class="music-lyric-viewport">
                                            <div class="music-lyric-track">
                                                <span ref="musicLyricTextRef" class="music-lyric-text">
                                                    {{ displayLyricLine }}
                                                </span>
                                                <span v-if="isLongDisplayLyricLine" class="music-lyric-text music-lyric-duplicate">
                                                    {{ displayLyricLine }}
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                </transition>
                            </div>
                        </div>
                    </transition>

                    <transition @enter="onInnerEnter" @leave="onInnerLeave" :css="false">
                        <div class="speed-box default-stats-box" v-show="isDefaultSpeedPrimary && !isMsgActive"
                            key="speed">
                            <div class="default-stat-item default-stat-compute">
                                <div class="default-stat-icon">
                                    <Cpu class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">CPU</div>
                                    <div class="default-stat-values">
                                        <span>{{ cpuUsage }}</span>
                                        <span v-if="hasCpuTemp">{{ cpuTemp }}</span>
                                        <span v-if="hasCpuFan">{{ cpuFan }}</span>
                                    </div>
                                </div>
                            </div>
                            <div class="default-stat-item default-stat-compute">
                                <div class="default-stat-icon">
                                    <CircuitBoard class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">显卡</div>
                                    <div class="default-stat-values">
                                        <span>{{ gpuUsage }}</span>
                                        <span>{{ gpuTemp }}</span>
                                        <span>{{ gpuFan }}</span>
                                    </div>
                                </div>
                            </div>
                            <div class="default-stat-item default-stat-memory">
                                <div class="default-stat-icon">
                                    <Database class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">显存</div>
                                    <div class="default-stat-values">
                                        <span>{{ gpuMemoryUsage }}</span>
                                        <span>{{ gpuMemory }}</span>
                                    </div>
                                </div>
                            </div>
                            <div class="default-stat-item default-stat-memory">
                                <div class="default-stat-icon">
                                    <MemoryStick class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">内存</div>
                                    <div class="default-stat-values">
                                        <span>{{ ramUsage }}</span>
                                        <span>{{ ramMemory }}</span>
                                    </div>
                                </div>
                            </div>
                            <div class="default-stat-item default-stat-network">
                                <div class="default-stat-icon">
                                    <Wifi class="metric-icon-glyph" :stroke-width="1.8" aria-hidden="true" />
                                </div>
                                <div class="default-stat-copy">
                                    <div class="default-stat-title">网络</div>
                                    <div class="default-stat-values network-values">
                                        <span :class="{ 'high-traffic': isHighDownload }">{{ downloadSpeed }}↓</span>
                                        <span :class="{ 'high-traffic': isHighUpload }">{{ uploadSpeed }}↑</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </transition>
                </div>

                <div v-if="isMusicPrimary && !isMsgActive" :class="['music-wave', { 'is-playing': isPlaying }]"
                    :style="musicWaveStyle">
                    <span></span>
                    <span></span>
                    <span></span>
                    <span></span>
                </div>
                <button v-if="isMusicPrimary && !isMsgActive" class="island-menu-button" title="菜单"
                    @mousedown.stop @click.stop="handleMenuButtonClick">
                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                        <circle cx="5" cy="12" r="1.7" />
                        <circle cx="12" cy="12" r="1.7" />
                        <circle cx="19" cy="12" r="1.7" />
                    </svg>
                </button>
                <div v-else-if="isMsgActive" class="msg-status-dot"></div>
                <div v-else-if="!isHardwarePrimary && !isDefaultSpeedPrimary" :class="['status-dot', networkStatus]">
                </div>
            </div>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick, type CSSProperties } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, currentMonitor, PhysicalPosition, LogicalPosition, PhysicalSize } from '@tauri-apps/api/window'; import { Menu, MenuItem } from '@tauri-apps/api/menu';
import { listen, emit } from '@tauri-apps/api/event';
import { CircuitBoard, Cpu, Database, MemoryStick, Wifi } from 'lucide-vue-next';

const isIslandVisible = ref(false);
const isMenuOpen = ref(false);
const isIslandEnabled = ref(localStorage.getItem('nsd_island_enabled') !== 'false');

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
    isIslandEnabled.value = true;
}

const AUTO_ACTIVITY_DEFAULTS_KEY = 'nsd_auto_activity_defaults_v1';
if (localStorage.getItem(AUTO_ACTIVITY_DEFAULTS_KEY) !== 'true') {
    localStorage.setItem('nsd_music_ctrl', 'true');
    localStorage.setItem('nsd_hardware_mon', 'false');
    localStorage.setItem(AUTO_ACTIVITY_DEFAULTS_KEY, 'true');
}

// 灵动岛自身的透明度变量（默认30）
const islandOpacity = ref(Number(localStorage.getItem('nsd_island_opacity') || '30'));

const islandTheme = ref(localStorage.getItem('nsd_island_theme') || 'system');
const systemPrefersDark = ref(window.matchMedia('(prefers-color-scheme: dark)').matches);
const resolvedIslandTheme = computed(() =>
    islandTheme.value === 'system'
        ? (systemPrefersDark.value ? 'black' : 'white')
        : islandTheme.value
);
const islandOpacityRatio = computed(() => clamp(islandOpacity.value / 100, 0, 1));
const islandAlpha = computed(() => Math.pow(islandOpacityRatio.value, 1 / 2.2));
// 修改后的 islandStyle
const islandStyle = computed<CSSProperties>(() => {
    const alpha = islandAlpha.value;
    const baseStyle = resolvedIslandTheme.value === 'white' ? {
        backgroundColor: `rgba(255, 255, 255, ${alpha})`,
        color: '#000000'
    } : {
        backgroundColor: `rgba(0, 0, 0, ${alpha})`,
        color: '#ffffff'
    };

    return {
        ...baseStyle,
        width: `${currentWidth.value}px`,
        height: `${currentHeight.value}px`,
        position: 'relative', // 改为相对定位或保留，由父级负责居中
    };
});

const coreContentStyle = computed(() => {
    const alpha = islandAlpha.value;
    if (resolvedIslandTheme.value === 'white') {
        return {
            backgroundColor: `rgba(255, 255, 255, ${alpha})`
        };
    }
    return {
        backgroundColor: `rgba(0, 0, 0, ${alpha})`
    };
});

const glowOpacity = computed(() => {
    return islandAlpha.value;
});

type RgbColor = { r: number; g: number; b: number };
type HslColor = { h: number; s: number; l: number };
type MusicStatus = {
    title: string;
    artist: string;
    sourceAppId?: string;
    thumbnailDataUri?: string | null;
    mediaKind?: string;
    isPlaying: boolean;
    positionMs?: number | null;
    durationMs?: number | null;
};

type SyncedLyricLine = {
    timeMs: number;
    text: string;
};

type SyncedLyrics = {
    source: string;
    lines: SyncedLyricLine[];
};

const UNKNOWN_ARTIST_LABELS = new Set(['未知歌手', 'unknown artist', 'unknown']);

function isUnknownArtistLabel(value: string) {
    const normalized = value.trim().toLowerCase();
    return !normalized || UNKNOWN_ARTIST_LABELS.has(normalized);
}

type HardwareStats = {
    cpuUsage: number;
    cpuTemperature?: number | null;
    cpuFanRpm?: number | null;
    usedMemory: number;
    totalMemory: number;
    gpuUsage?: number | null;
    gpuFanRpm?: number | null;
    gpuFanSpeedPercent?: number | null;
    gpuMemoryUsage?: number | null;
    gpuMemoryUsedMb?: number | null;
    gpuMemoryTotalMb?: number | null;
    gpuTemperature?: number | null;
};

const DEFAULT_BORDER_COLORS: RgbColor[] = [
    { r: 238, g: 86, b: 72 },
    { r: 190, g: 70, b: 76 },
    { r: 116, g: 62, b: 96 },
    { r: 72, g: 86, b: 116 },
    { r: 156, g: 82, b: 64 },
    { r: 238, g: 112, b: 78 },
];

const borderColors = ref<string[]>(DEFAULT_BORDER_COLORS.map((color) => rgbToCss(color)));

const albumBorderStyle = computed<CSSProperties>(() => ({
    opacity: glowOpacity.value,
    backgroundImage: (isDefaultSpeedPrimary.value || isHardwarePrimary.value)
        ? buildWideIslandGlowGradient(borderColors.value)
        : buildBorderGradient(borderColors.value),
}));

const splitMusicBubbleStyle = computed<CSSProperties>(() => {
    if (!isGlowBorderEnabled.value) {
        return coreContentStyle.value;
    }

    const fill = coreContentStyle.value.backgroundColor ?? 'rgba(0, 0, 0, 0.78)';
    return {
        ...coreContentStyle.value,
        backgroundImage: `linear-gradient(${fill}, ${fill}), ${buildBorderGradient(borderColors.value, glowOpacity.value)}`,
        backgroundOrigin: 'border-box',
        backgroundClip: 'padding-box, border-box',
    };
});

const splitCoreStyle = computed<CSSProperties>(() => {
    if (!isSplitActivity.value || !isGlowBorderEnabled.value) {
        return coreContentStyle.value;
    }

    const fill = coreContentStyle.value.backgroundColor ?? `rgba(0, 0, 0, ${islandAlpha.value})`;
    return {
        ...coreContentStyle.value,
        border: '2px solid transparent',
        backgroundImage: `linear-gradient(${fill}, ${fill}), ${buildBorderGradient(borderColors.value, glowOpacity.value)}`,
        backgroundOrigin: 'border-box',
        backgroundClip: 'padding-box, border-box',
    };
});

function clamp(value: number, min: number, max: number) {
    return Math.min(max, Math.max(min, value));
}

function rgbToCss({ r, g, b }: RgbColor) {
    return `rgb(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)})`;
}

function rgbToHsl({ r, g, b }: RgbColor): HslColor {
    const rn = r / 255;
    const gn = g / 255;
    const bn = b / 255;
    const max = Math.max(rn, gn, bn);
    const min = Math.min(rn, gn, bn);
    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (max !== min) {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
        switch (max) {
            case rn:
                h = (gn - bn) / d + (gn < bn ? 6 : 0);
                break;
            case gn:
                h = (bn - rn) / d + 2;
                break;
            default:
                h = (rn - gn) / d + 4;
        }
        h *= 60;
    }

    return { h, s, l };
}

function hslToRgb({ h, s, l }: HslColor): RgbColor {
    const hue = (((h % 360) + 360) % 360) / 360;
    if (s === 0) {
        const v = l * 255;
        return { r: v, g: v, b: v };
    }

    const hueToRgb = (p: number, q: number, t: number) => {
        let tt = t;
        if (tt < 0) tt += 1;
        if (tt > 1) tt -= 1;
        if (tt < 1 / 6) return p + (q - p) * 6 * tt;
        if (tt < 1 / 2) return q;
        if (tt < 2 / 3) return p + (q - p) * (2 / 3 - tt) * 6;
        return p;
    };

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    return {
        r: hueToRgb(p, q, hue + 1 / 3) * 255,
        g: hueToRgb(p, q, hue) * 255,
        b: hueToRgb(p, q, hue - 1 / 3) * 255,
    };
}

function tuneForGlow(color: RgbColor) {
    const hsl = rgbToHsl(color);
    return rgbToCss(hslToRgb({
        h: hsl.h,
        s: clamp(Math.max(hsl.s, 0.46), 0.32, 0.86),
        l: clamp(hsl.l, 0.36, 0.66),
    }));
}

function tuneForDashboardGlow(color: RgbColor) {
    const hsl = rgbToHsl(color);
    return rgbToCss(hslToRgb({
        h: hsl.h,
        s: clamp(hsl.s * 0.86, 0.34, 0.68),
        l: clamp(hsl.l + 0.06, 0.42, 0.68),
    }));
}

function colorDistance(a: RgbColor, b: RgbColor) {
    return Math.sqrt(
        Math.pow(a.r - b.r, 2) +
        Math.pow(a.g - b.g, 2) +
        Math.pow(a.b - b.b, 2)
    );
}

function expandPalette(colors: RgbColor[]) {
    if (colors.length === 0) return DEFAULT_BORDER_COLORS.map((color) => rgbToCss(color));

    const expanded = colors.slice(0, 3);
    const baseHsl = rgbToHsl(expanded[0]);
    const hueOffsets = [12, -12, 24, -24, 36, -36];

    while (expanded.length < 6) {
        const offset = hueOffsets[expanded.length - 1] ?? expanded.length * 36;
        expanded.push(hslToRgb({
            h: baseHsl.h + offset,
            s: clamp(Math.max(baseHsl.s, 0.42), 0.3, 0.72),
            l: clamp(baseHsl.l + (expanded.length % 2 === 0 ? 0.05 : -0.04), 0.34, 0.62),
        }));
    }

    return expanded.map(tuneForGlow);
}

function parseCssRgb(color: string): RgbColor | null {
    const match = color.match(/rgba?\(([^)]+)\)/i);
    if (!match) return null;

    const parts = match[1]
        .split(',')
        .map((part) => Number.parseFloat(part.trim()))
        .filter((value) => Number.isFinite(value));

    if (parts.length < 3) return null;
    return { r: parts[0], g: parts[1], b: parts[2] };
}

function mixRgb(a: RgbColor, b: RgbColor, amount: number): RgbColor {
    const t = clamp(amount, 0, 1);
    return {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    };
}

function buildWideIslandPalette(colors: string[]) {
    const parsed = colors
        .map(parseCssRgb)
        .filter((color): color is RgbColor => Boolean(color));
    const source = parsed.length > 0 ? parsed : DEFAULT_BORDER_COLORS;
    const primary = source[0];
    const secondary = source.find((color) => colorDistance(primary, color) > 46) ?? source[1] ?? primary;
    const anchor = mixRgb(primary, { r: 28, g: 22, b: 28 }, 0.32);

    return [
        tuneForDashboardGlow(primary),
        rgbToCss(anchor),
        tuneForDashboardGlow(secondary),
    ];
}

function colorWithAlpha(color: string, alpha: number) {
    const match = color.match(/rgba?\(([^)]+)\)/i);
    if (!match) return color;

    const parts = match[1]
        .split(',')
        .map((part) => Number.parseFloat(part.trim()))
        .filter((value) => Number.isFinite(value));

    if (parts.length < 3) return color;
    return `rgba(${Math.round(parts[0])}, ${Math.round(parts[1])}, ${Math.round(parts[2])}, ${clamp(alpha, 0, 1)})`;
}

function buildBorderGradient(colors: string[], alpha = 1) {
    const palette = colors.length >= 6 ? colors : DEFAULT_BORDER_COLORS.map((color) => rgbToCss(color));
    const appliedPalette = alpha < 1 ? palette.map((color) => colorWithAlpha(color, alpha)) : palette;
    return `conic-gradient(from 0deg, ${appliedPalette[0]} 0deg, ${appliedPalette[1]} 60deg, ${appliedPalette[2]} 120deg, ${appliedPalette[3]} 180deg, ${appliedPalette[4]} 240deg, ${appliedPalette[5]} 300deg, ${appliedPalette[0]} 360deg)`;
}

function buildWideIslandGlowGradient(colors: string[], alpha = 1) {
    const palette = buildWideIslandPalette(colors);
    const appliedPalette = alpha < 1 ? palette.map((color) => colorWithAlpha(color, alpha)) : palette;
    return `linear-gradient(105deg, ${appliedPalette[0]} 0%, ${appliedPalette[1]} 42%, ${appliedPalette[2]} 72%, ${appliedPalette[1]} 100%)`;
}

function relativeLuminance(color: RgbColor) {
    const channel = (value: number) => {
        const normalized = value / 255;
        return normalized <= 0.03928
            ? normalized / 12.92
            : Math.pow((normalized + 0.055) / 1.055, 2.4);
    };

    return 0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b);
}

function contrastRatio(a: RgbColor, b: RgbColor) {
    const bright = Math.max(relativeLuminance(a), relativeLuminance(b));
    const dark = Math.min(relativeLuminance(a), relativeLuminance(b));
    return (bright + 0.05) / (dark + 0.05);
}

function chooseWaveBaseColor(colors: string[]) {
    const parsed = colors
        .map(parseCssRgb)
        .filter((color): color is RgbColor => Boolean(color));
    const palette = parsed.length > 0 ? parsed : DEFAULT_BORDER_COLORS;
    const bg = resolvedIslandTheme.value === 'white'
        ? { r: 255, g: 255, b: 255 }
        : { r: 0, g: 0, b: 0 };

    return palette
        .slice()
        .sort((a, b) => contrastRatio(b, bg) - contrastRatio(a, bg))[0] ?? DEFAULT_BORDER_COLORS[0];
}

function tuneWaveColor(color: RgbColor) {
    const hsl = rgbToHsl(color);
    const onLightBackground = resolvedIslandTheme.value === 'white';

    return hslToRgb({
        h: hsl.h,
        s: clamp(Math.max(hsl.s, 0.58), 0.48, 0.92),
        l: onLightBackground
            ? clamp(Math.min(hsl.l, 0.38), 0.22, 0.42)
            : clamp(Math.max(hsl.l, 0.62), 0.58, 0.78),
    });
}

function rgbaFromRgb(color: RgbColor, alpha: number) {
    return `rgba(${Math.round(color.r)}, ${Math.round(color.g)}, ${Math.round(color.b)}, ${clamp(alpha, 0, 1)})`;
}

const musicWaveStyle = computed<CSSProperties>(() => {
    const base = tuneWaveColor(chooseWaveBaseColor(borderColors.value));
    const hsl = rgbToHsl(base);
    const top = hslToRgb({
        h: hsl.h,
        s: clamp(hsl.s + 0.08, 0.5, 0.96),
        l: resolvedIslandTheme.value === 'white'
            ? clamp(hsl.l + 0.08, 0.28, 0.52)
            : clamp(hsl.l + 0.1, 0.64, 0.84),
    });
    const bottom = hslToRgb({
        h: hsl.h,
        s: hsl.s,
        l: resolvedIslandTheme.value === 'white'
            ? clamp(hsl.l - 0.08, 0.18, 0.36)
            : clamp(hsl.l - 0.18, 0.42, 0.68),
    });

    return {
        '--music-wave-top': rgbToCss(top),
        '--music-wave-bottom': rgbToCss(bottom),
        '--music-wave-glow': rgbaFromRgb(base, resolvedIslandTheme.value === 'white' ? 0.28 : 0.42),
    } as CSSProperties;
});

async function loadImage(url: string) {
    const image = new Image();
    const loaded = new Promise<void>((resolve, reject) => {
        image.onload = () => resolve();
        image.onerror = () => reject(new Error('封面加载失败'));
    });

    if (!url.startsWith('data:')) {
        image.crossOrigin = 'anonymous';
    }
    image.src = url;

    try {
        await image.decode();
    } catch {
        await loaded;
    }

    return image;
}

async function extractBorderColorsFromCover(url: string) {
    const image = await loadImage(url);
    const canvas = document.createElement('canvas');
    const size = 48;
    canvas.width = size;
    canvas.height = size;

    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) return [];

    ctx.drawImage(image, 0, 0, size, size);
    const pixels = ctx.getImageData(0, 0, size, size).data;
    const buckets = new Map<string, { color: RgbColor; count: number; score: number }>();

    for (let i = 0; i < pixels.length; i += 4) {
        const alpha = pixels[i + 3];
        if (alpha < 180) continue;

        const color = { r: pixels[i], g: pixels[i + 1], b: pixels[i + 2] };
        const hsl = rgbToHsl(color);
        if (hsl.l < 0.06 || hsl.l > 0.94) continue;

        const key = [
            Math.round(color.r / 24) * 24,
            Math.round(color.g / 24) * 24,
            Math.round(color.b / 24) * 24,
        ].join(',');
        const bucketColor = {
            r: Math.round(color.r / 24) * 24,
            g: Math.round(color.g / 24) * 24,
            b: Math.round(color.b / 24) * 24,
        };
        const bucket = buckets.get(key) ?? { color: bucketColor, count: 0, score: 0 };
        bucket.count += 1;
        bucket.score += (0.35 + hsl.s * 1.8) * (1 - Math.abs(hsl.l - 0.52) * 0.75);
        buckets.set(key, bucket);
    }

    const sorted = [...buckets.values()].sort((a, b) => b.count * b.score - a.count * a.score);
    const selected: RgbColor[] = [];

    for (const item of sorted) {
        if (selected.every((color) => colorDistance(color, item.color) > 58)) {
            selected.push(item.color);
        }
        if (selected.length >= 3) break;
    }

    return expandPalette(selected);
}

const uploadSpeed = ref('0 KB/s');
const downloadSpeed = ref('0 KB/s');

// 记录当前是否属于大流量状态
const isHighDownload = ref(false);
const isHighUpload = ref(false);

// 网络状态指示灯：good(绿), warning(黄), error(红)
const networkStatus = ref<'good' | 'warning' | 'error'>('good');

const DASHBOARD_MODES_ENABLED = true;

const isHardwareMonEnabled = ref(localStorage.getItem('nsd_hardware_mon') === 'true');
const isLyricsEnabled = ref(localStorage.getItem('nsd_show_lyrics') !== 'false');
const showCpuMetric = ref(localStorage.getItem('nsd_show_cpu') !== 'false');
const showGpuMetric = ref(localStorage.getItem('nsd_show_gpu') !== 'false');
const showMemoryMetric = ref(localStorage.getItem('nsd_show_memory') !== 'false');
const showNetworkMetric = ref(localStorage.getItem('nsd_show_network') !== 'false');
const cpuUsage = ref('0%');
const cpuTemp = ref('--°C');
const cpuFan = ref('--');
const gpuUsage = ref('0%');
const gpuTemp = ref('--°C');
const gpuFan = ref('--');
const gpuMemoryUsage = ref('--');
const gpuMemory = ref('--/--');
const ramUsage = ref('0%');
const ramMemory = ref('--/--');
const hasCpuTemp = computed(() => cpuTemp.value !== '--°C');
const hasCpuFan = computed(() => cpuFan.value !== '--');

// 音乐控制功能开关
const isMusicCtlEnabled = ref(localStorage.getItem('nsd_music_ctrl') !== 'false');
const isPlaying = ref(false);
const hasMusicSession = ref(false);
const currentMusicSourceAppId = ref('');
const currentSongTitle = ref('');
const currentSongArtist = ref('');
const currentMediaKind = ref<'music' | 'browser'>('music');
const currentCoverSource = ref('None');
const currentProgressSource = ref('None');
const musicPositionMs = ref<number | null>(null);
const musicDurationMs = ref<number | null>(null);
const musicPositionSyncedAt = ref(0);
let lastRawMusicPositionMs: number | null = null;
let lastRawMusicPositionChangedAt = 0;
let lastAcceptedMusicPositionMs: number | null = null;
const lyricNowMs = ref<number | null>(null);
const fallbackLyricPositionMs = ref<number | null>(null);
let fallbackLyricSyncedAt = 0;
const lyricLines = ref<SyncedLyricLine[]>([]);
const lyricsSource = ref('');
const lyricOffsetMs = ref(Number(localStorage.getItem('nsd_lyric_offset_ms') || '0'));
let isClickingToggle = false;
// 流光边框默认状态完全镜像音乐控制器（只要音乐控制器开着它就开，关了就一起关）
const isGlowBorderEnabled = ref(localStorage.getItem('nsd_glow_border') !== 'false');

const coverUrl = ref('');
const coverCache = new Map<string, string>();
const lyricsCache = new Map<string, SyncedLyricLine[] | null>();
let lyricsRequestId = 0;
let paletteRequestId = 0;
let paletteTransitionFrame: number | null = null;

function normalizePalette(colors: string[]) {
    const fallback = DEFAULT_BORDER_COLORS.map((color) => rgbToCss(color));
    return Array.from({ length: 6 }, (_, index) => colors[index] ?? fallback[index] ?? fallback[0]);
}

function animateBorderColors(targetColors: string[]) {
    const from = normalizePalette(borderColors.value)
        .map(parseCssRgb)
        .map((color, index) => color ?? DEFAULT_BORDER_COLORS[index] ?? DEFAULT_BORDER_COLORS[0]);
    const to = normalizePalette(targetColors)
        .map(parseCssRgb)
        .map((color, index) => color ?? DEFAULT_BORDER_COLORS[index] ?? DEFAULT_BORDER_COLORS[0]);

    if (paletteTransitionFrame !== null) {
        cancelAnimationFrame(paletteTransitionFrame);
        paletteTransitionFrame = null;
    }

    const start = performance.now();
    const duration = 720;

    const animate = (time: number) => {
        const progress = clamp((time - start) / duration, 0, 1);
        const eased = 1 - Math.pow(1 - progress, 3);

        borderColors.value = from.map((color, index) => rgbToCss(mixRgb(color, to[index], eased)));

        if (progress < 1) {
            paletteTransitionFrame = requestAnimationFrame(animate);
        } else {
            borderColors.value = normalizePalette(targetColors);
            paletteTransitionFrame = null;
        }
    };

    paletteTransitionFrame = requestAnimationFrame(animate);
}

watch(coverUrl, async (url) => {
    const requestId = ++paletteRequestId;

    if (!url) {
        animateBorderColors(DEFAULT_BORDER_COLORS.map((color) => rgbToCss(color)));
        return;
    }

    try {
        const colors = await extractBorderColorsFromCover(url);
        if (requestId === paletteRequestId && colors.length > 0) {
            animateBorderColors(colors);
        }
    } catch (error) {
        if (requestId === paletteRequestId) {
            animateBorderColors(DEFAULT_BORDER_COLORS.map((color) => rgbToCss(color)));
        }
        console.warn('专辑封面取色失败，已使用默认流光边框:', error);
    }
});

function isValidMs(value: unknown): value is number {
    return typeof value === 'number' && Number.isFinite(value) && value >= 0;
}

function syncMusicTimeline(positionMs?: number | null, durationMs?: number | null, playing = false) {
    musicDurationMs.value = isValidMs(durationMs) && durationMs > 0 ? durationMs : null;
    const now = performance.now();

    if (isValidMs(positionMs)) {
        const projected = getProjectedMusicPositionMs();
        const previousRaw = lastRawMusicPositionMs;
        const previousAccepted = lastAcceptedMusicPositionMs;
        const rawDelta = previousRaw === null ? 0 : positionMs - previousRaw;
        const projectionDelta = projected === null ? 0 : positionMs - projected;
        const acceptedDelta = previousAccepted === null ? 0 : positionMs - previousAccepted;
        const rawChanged = previousRaw === null || Math.abs(rawDelta) > 350;
        const looksLikeSeek = rawChanged && (Math.abs(projectionDelta) > 2500 || Math.abs(acceptedDelta) > 2500);

        if (rawChanged) {
            lastRawMusicPositionMs = positionMs;
            lastRawMusicPositionChangedAt = now;
        }

        const rawLooksFrozen = playing
            && !rawChanged
            && now - lastRawMusicPositionChangedAt > 900
            && projected !== null
            && !looksLikeSeek
            && projected > positionMs + 650;

        const timelinePosition = rawLooksFrozen ? projected : positionMs;
        musicPositionMs.value = Math.min(timelinePosition, musicDurationMs.value ?? timelinePosition);
        musicPositionSyncedAt.value = now;
        lastAcceptedMusicPositionMs = musicPositionMs.value;
        fallbackLyricPositionMs.value = null;
        lyricNowMs.value = musicPositionMs.value;
        return;
    }

    musicPositionMs.value = null;
    if (lyricLines.value.length > 0) {
        if (fallbackLyricPositionMs.value === null) {
            fallbackLyricPositionMs.value = getFallbackLyricStartMs(lyricLines.value);
            fallbackLyricSyncedAt = performance.now();
        }
        lyricNowMs.value = getProjectedMusicPositionMs();
    } else {
        lyricNowMs.value = null;
    }
}

function getProjectedMusicPositionMs() {
    let position: number | null = null;
    let syncedAt = 0;

    if (musicPositionMs.value !== null) {
        position = musicPositionMs.value;
        syncedAt = musicPositionSyncedAt.value;
    } else if (fallbackLyricPositionMs.value !== null) {
        position = fallbackLyricPositionMs.value;
        syncedAt = fallbackLyricSyncedAt;
    }

    if (position === null) return null;

    if (isPlaying.value && syncedAt > 0) {
        position += performance.now() - syncedAt;
    }

    if (musicDurationMs.value !== null) {
        position = Math.min(position, musicDurationMs.value);
    }

    return Math.max(0, position);
}

function refreshLyricClock() {
    lyricNowMs.value = getProjectedMusicPositionMs();
}

function buildLyricsKey(songName: string, artistName: string, durationMs?: number | null) {
    const safe = (value: string) => value.trim().toLowerCase().replace(/\s+/g, ' ');
    const durationSeconds = isValidMs(durationMs) ? Math.round(durationMs / 1000) : 0;
    return `${safe(songName)}::${safe(artistName)}::${durationSeconds}`;
}

function isLyricCreditLine(text: string) {
    const normalized = text.trim().replace(/^[：:\s]+|[：:\s]+$/g, '').toLowerCase();
    if (!normalized) return true;

    const compact = normalized.replace(/\s+/g, '');
    const cjkPrefixes = [
        '作词', '作詞', '词', '詞', '作曲', '曲', '编曲', '編曲', '制作人', '制作',
        '监制', '監製', '出品', '发行', '發行', '录音', '錄音', '混音', '母带',
        '母帶', '和声', '和聲', '配唱', '统筹', '統籌', '企划', '企劃', '吉他',
        '贝斯', '貝斯', '鼓', '键盘', '鍵盤', '弦乐', '弦樂', '人声', '人聲',
    ];
    if (cjkPrefixes.some((prefix) => {
        if (!compact.startsWith(prefix)) return false;
        const next = compact.charAt(prefix.length);
        return !next || ['：', ':', '/', '-', '_', '|', '丨'].includes(next);
    })) {
        return true;
    }

    return [
        'lyrics by', 'lyricist', 'written by', 'composer', 'composed by', 'arranger',
        'arranged by', 'producer', 'produced by', 'mixing', 'mixed by', 'mastering',
        'mastered by', 'vocal', 'guitar', 'bass', 'drums', 'keyboard', 'op:', 'sp:',
    ].some((prefix) => normalized.startsWith(prefix));
}
void isLyricCreditLine;

function sanitizeLyricLines(lines?: SyncedLyricLine[]) {
    return (lines ?? [])
        .filter((line) =>
            isValidMs(line.timeMs)
            && typeof line.text === 'string'
            && line.text.trim().length > 0
        )
        .map((line) => ({ timeMs: line.timeMs, text: line.text.trim() }))
        .sort((a, b) => a.timeMs - b.timeMs);
}

function getFallbackLyricStartMs(lines: SyncedLyricLine[]) {
    const first = lines[0]?.timeMs ?? 0;
    return Math.max(0, first - 900);
}

function containsCjkText(value: string) {
    return /[\u3400-\u9fff\u3040-\u30ff\uac00-\ud7af]/.test(value);
}

const lyricLeadMs = computed(() => {
    const hasCjk = containsCjkText(currentSongTitle.value)
        || containsCjkText(currentSongArtist.value)
        || lyricLines.value.slice(0, 12).some((line) => containsCjkText(line.text));
    return (hasCjk ? 650 : 320) + lyricOffsetMs.value;
});

function clearLyrics() {
    lyricsRequestId += 1;
    lyricLines.value = [];
    lyricsSource.value = '';
    lyricNowMs.value = null;
    fallbackLyricPositionMs.value = null;
    fallbackLyricSyncedAt = 0;
    lastRawMusicPositionMs = null;
    lastRawMusicPositionChangedAt = 0;
    lastAcceptedMusicPositionMs = null;
}

async function loadSyncedLyrics(songName: string, artistName: string, durationMs?: number | null) {
    const key = buildLyricsKey(songName, artistName, durationMs);
    const cached = lyricsCache.get(key);
    if (lyricsCache.has(key)) {
        lyricLines.value = cached ? cached.slice() : [];
        lyricsSource.value = cached ? 'LRCLIB' : '';
        if (lyricLines.value.length > 0 && musicPositionMs.value === null && fallbackLyricPositionMs.value === null) {
            fallbackLyricPositionMs.value = getFallbackLyricStartMs(lyricLines.value);
            fallbackLyricSyncedAt = performance.now();
        }
        refreshLyricClock();
        return;
    }

    const requestId = ++lyricsRequestId;
    lyricLines.value = [];
    lyricsSource.value = '';

    try {
        const result = await invoke<SyncedLyrics | null>('fetch_synced_lyrics', {
            songName,
            artistName,
            durationMs: isValidMs(durationMs) ? Math.round(durationMs) : null,
        });
        const lines = sanitizeLyricLines(result?.lines);

        if (lyricsCache.size > 80) lyricsCache.clear();
        lyricsCache.set(key, lines.length > 0 ? lines : null);

        if (requestId !== lyricsRequestId) return;
        lyricLines.value = lines;
        lyricsSource.value = lines.length > 0 ? (result?.source || 'LRCLIB') : '';
        if (lines.length > 0 && musicPositionMs.value === null && fallbackLyricPositionMs.value === null) {
            fallbackLyricPositionMs.value = getFallbackLyricStartMs(lines);
            fallbackLyricSyncedAt = performance.now();
        }
        refreshLyricClock();
    } catch (error) {
        lyricsCache.set(key, null);
        if (requestId === lyricsRequestId) {
            lyricLines.value = [];
            lyricsSource.value = '';
        }
        console.warn('同步歌词获取失败，已降级为仅显示歌名:', error);
    }
}

const activeLyricLine = computed(() => {
    if (lyricLines.value.length === 0 || lyricNowMs.value === null) {
        return '';
    }

    const target = lyricNowMs.value + lyricLeadMs.value;
    if (target < lyricLines.value[0].timeMs - 1200) {
        return '';
    }

    let left = 0;
    let right = lyricLines.value.length - 1;
    while (left <= right) {
        const mid = Math.floor((left + right) / 2);
        if (lyricLines.value[mid].timeMs <= target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }

    return lyricLines.value[Math.max(0, right)]?.text ?? '';
});

const upcomingLyricLine = computed(() => {
    if (activeLyricLine.value || lyricLines.value.length === 0) {
        return '';
    }

    const firstLine = lyricLines.value[0];
    if (lyricNowMs.value === null) {
        return firstLine.text;
    }
    return lyricNowMs.value < firstLine.timeMs ? firstLine.text : '';
});

const displayLyricLine = computed(() => activeLyricLine.value || upcomingLyricLine.value);
const hasDisplayLyricLine = computed(() => isLyricsEnabled.value && displayLyricLine.value.length > 0);
const isUpcomingLyricLine = computed(() => !activeLyricLine.value && upcomingLyricLine.value.length > 0);
const musicLyricViewportRef = ref<HTMLElement | null>(null);
const musicLyricTextRef = ref<HTMLElement | null>(null);
const isLyricLineOverflowing = ref(false);
let lyricMeasureFrame: number | null = null;
const lyricDisplayWeight = computed(() =>
    Array.from(displayLyricLine.value.trim()).reduce((weight, ch) => {
        const code = ch.codePointAt(0) ?? 0;
        return weight + (code > 0xff ? 2 : 1);
    }, 0)
);
const isLongDisplayLyricLine = computed(() =>
    isLyricLineOverflowing.value || lyricDisplayWeight.value > 28
);

function scheduleLyricLineMeasure() {
    if (lyricMeasureFrame !== null) {
        cancelAnimationFrame(lyricMeasureFrame);
        lyricMeasureFrame = null;
    }

    nextTick(() => {
        lyricMeasureFrame = requestAnimationFrame(() => {
            lyricMeasureFrame = null;
            const viewport = musicLyricViewportRef.value;
            const lyric = musicLyricTextRef.value;
            if (!viewport || !lyric || !hasDisplayLyricLine.value || !isMusicPrimary.value || !showInfo.value) {
                isLyricLineOverflowing.value = false;
                return;
            }

            isLyricLineOverflowing.value = lyric.scrollWidth > viewport.clientWidth + 4;
        });
    });
}

watch(isPlaying, (_playing, wasPlaying) => {
    if (fallbackLyricPositionMs.value === null) return;

    const now = performance.now();
    if (wasPlaying && fallbackLyricSyncedAt > 0) {
        fallbackLyricPositionMs.value += now - fallbackLyricSyncedAt;
    }
    fallbackLyricSyncedAt = now;
    refreshLyricClock();
});

// 记录是否开启了置于任务栏
const isPinnedToTaskbar = ref(localStorage.getItem('nsd_pin_taskbar') === 'true');

// 记录消息模式开关状态
const isMsgModeEnabled = ref(localStorage.getItem('nsd_msg_mode') !== 'false');
let islandThemeMedia: MediaQueryList | null = null;
const handleIslandSystemThemeUpdate = (event: MediaQueryListEvent) => {
    systemPrefersDark.value = event.matches;
};

// 计算并吸附到左下角的方法
const snapToBottomLeft = async () => {
    try {
        const appWindow = getCurrentWindow();
        await new Promise((resolve) => setTimeout(resolve, 150));
        const monitor = await currentMonitor();

        if (monitor) {
            const scaleFactor = window.devicePixelRatio;

            const WINDOW_INIT_WIDTH = currentWidth.value;
            const WINDOW_INIT_HEIGHT = currentHeight.value;
            await appWindow.setSize(new PhysicalSize(
                getWindowPhysicalWidth(WINDOW_INIT_WIDTH, scaleFactor),
                getWindowPhysicalHeight(WINDOW_INIT_HEIGHT, scaleFactor)
            ));

            const monitorLeftPhysical = monitor.position.x;
            const monitorTopPhysical = monitor.position.y;
            // 恢复使用 Tauri 最底层的硬件真实分辨率（绝对不会缩水）
            const monitorHeightPhysical = monitor.size.height;

            // X坐标: 屏幕最左侧 + 10px的边距
            const x = monitorLeftPhysical + ((10 - WINDOW_SAFE_PADDING_X) * scaleFactor);
            // Y坐标: 物理最底部 - 窗口高度 - 3px微调
            const y = monitorTopPhysical + monitorHeightPhysical - ((WINDOW_INIT_HEIGHT + 3) * scaleFactor);

            // 【终极绝杀核心】：绕过 Windows 系统的任务栏防遮挡机制
            // 在强制覆盖任务栏坐标之前，先隐身！
            await appWindow.hide();

            await appWindow.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));

            // 移动完成后，瞬间现身，生米煮成熟饭，Windows 也拦不住了！
            await appWindow.show();

            trackedPhysicalX = Math.round(x);
            trackedPhysicalY = Math.round(y);
        }
    } catch (error) {
        console.error('停靠左下角失败:', error);
    }
};

const togglePlay = async () => {
    isPlaying.value = !isPlaying.value;
    isClickingToggle = true; // 锁定 UI 不接受后端轮询的覆盖
    try {
        await invoke('control_system_media', { action: 'play_pause' });
    } catch (err) {
        console.error(err);
    }
    setTimeout(() => {
        // 👇 关键修改点：从 1500 延长到 2500。
        // 这意味着你点完暂停的 2.5 秒内，图标死死卡住绝对不动。
        // 2.5 秒后解锁时，后端的 2 秒静音宽限期必定已过，后端就会传来真实的 false（停止输出分贝）！
        isClickingToggle = false;
    }, 2500);
};

const prevTrack = async () => {
    await invoke('control_system_media', { action: 'prev' });
};

const nextTrack = async () => {
    await invoke('control_system_media', { action: 'next' });
};

const openMusicApp = async () => {
    try {
        await invoke('open_music_app', { sourceAppId: currentMusicSourceAppId.value });
    } catch (err) {
        console.error('打开音乐播放器失败:', err);
    }
};

// 核心同步函数：塞入到你的 fetchSpeedStats 同一频次的定时器中
const syncMusicStatus = async () => {
    try {
        // 1. 调用 Rust 提取系统媒体标题、播放状态和系统媒体进度
        const res = await invoke<MusicStatus | null>('fetch_music_info');

        if (res) {
            hasMusicSession.value = true;
            const song = res.title;
            const artist = res.artist;
            const playing = res.isPlaying;
            const sourceAppId = res.sourceAppId ?? '';
            const mediaKind = res.mediaKind ?? 'music';
            const isBrowserMedia = mediaKind === 'browser';

            if (isBrowserMedia) {
                hasMusicSession.value = false;
                currentMusicSourceAppId.value = '';
                currentMediaKind.value = 'music';
                currentCoverSource.value = 'None';
                currentProgressSource.value = 'None';
                currentSongTitle.value = '';
                currentSongArtist.value = '';
                currentTrackInfo.value = '未在播放歌曲 - 音乐软件';
                currentTrackCacheKey.value = '';
                isPlaying.value = false;
                coverUrl.value = '';
                musicPositionMs.value = null;
                musicDurationMs.value = null;
                clearLyrics();
                return;
            }

            currentMusicSourceAppId.value = sourceAppId;
            currentMediaKind.value = isBrowserMedia ? 'browser' : 'music';
            currentSongTitle.value = song;
            currentSongArtist.value = artist;

            // 拼接新的歌曲信息
            const newTrackInfo = isBrowserMedia || isUnknownArtistLabel(artist)
                ? song
                : `${song} - ${artist}`;
            const newTrackCacheKey = `${mediaKind}::${sourceAppId || 'unknown'}::${song}::${artist}`;

            if (currentTrackCacheKey.value !== newTrackCacheKey) {
                currentTrackCacheKey.value = newTrackCacheKey;
                currentTrackInfo.value = newTrackInfo;
                clearLyrics();

                // 优先读取缓存
                if (res.thumbnailDataUri) {
                    coverUrl.value = res.thumbnailDataUri;
                    currentCoverSource.value = 'Windows media thumbnail';
                    if (coverCache.size > 50) coverCache.clear();
                    coverCache.set(newTrackCacheKey, res.thumbnailDataUri);
                } else if (coverCache.has(newTrackCacheKey)) {
                    coverUrl.value = coverCache.get(newTrackCacheKey)!;
                    currentCoverSource.value = 'Local cache';
                } else if (isBrowserMedia) {
                    coverUrl.value = '';
                    currentCoverSource.value = 'Browser thumbnail missing';
                } else {
                    try {
                        const realCoverUrl = await invoke<string>('get_random_cover_url', {
                            songName: song,
                            artistName: artist,
                            sourceAppId
                        });
                        coverUrl.value = realCoverUrl;
                        currentCoverSource.value = 'Platform cover API';
                        // 写入缓存，最多缓存 50 首防止内存溢出
                        if (coverCache.size > 50) coverCache.clear();
                        coverCache.set(newTrackCacheKey, realCoverUrl);
                    } catch (coverErr) {
                        console.error('所有封面源均获取失败:', coverErr);
                        // 使用本地图标或纯色背景，不要再用外部 URL 作为错误兜底
                        coverUrl.value = '';
                        currentCoverSource.value = 'Cover API failed';
                    }
                }

                if (!isBrowserMedia) {
                    loadSyncedLyrics(song, artist, res.durationMs ?? null).catch(console.error);
                }
            }

            if (!isClickingToggle) {
                isPlaying.value = playing;
            }
            currentProgressSource.value = (res.positionMs ?? res.durationMs) != null
                ? 'Windows media session'
                : 'No real progress';
            syncMusicTimeline(res.positionMs ?? null, res.durationMs ?? null, playing);
        } else {
            // 没检测到播放时，清空状态
            hasMusicSession.value = false;
            currentMusicSourceAppId.value = '';
            currentMediaKind.value = 'music';
            currentCoverSource.value = 'None';
            currentProgressSource.value = 'None';
            currentSongTitle.value = '';
            currentSongArtist.value = '';
            currentTrackInfo.value = '未在播放歌曲 - 音乐软件';
            currentTrackCacheKey.value = '';
            isPlaying.value = false;
            coverUrl.value = ''; // 没歌时清空，显示默认的优美渐变色
            musicPositionMs.value = null;
            musicDurationMs.value = null;
            clearLyrics();
        }
    } catch (err) {
        console.error('音乐信息获取失败:', err);
        hasMusicSession.value = false;
        currentMediaKind.value = 'music';
        currentCoverSource.value = 'None';
        currentProgressSource.value = 'None';
        currentSongTitle.value = '';
        currentSongArtist.value = '';
        currentTrackCacheKey.value = '';
        isPlaying.value = false;
        coverUrl.value = '';
        musicPositionMs.value = null;
        musicDurationMs.value = null;
        clearLyrics();
    } finally {
        await syncIslandVisibility();
    }
};

const showInfo = ref(false);
const currentTrackInfo = ref('未在播放歌曲 - 未知歌手'); // 默认显示内容
const currentTrackCacheKey = ref('');
const musicTitleViewportRef = ref<HTMLElement | null>(null);
const musicTitleTextRef = ref<HTMLElement | null>(null);
const isTrackTitleOverflowing = ref(false);
let titleMeasureFrame: number | null = null;
const isLongTrackTitle = computed(() => isTrackTitleOverflowing.value);
const isMusicPrimary = computed(() => isMusicCtlEnabled.value && hasMusicSession.value);
const visibleHardwareMetricCount = computed(() =>
    Number(showCpuMetric.value)
    + Number(showGpuMetric.value) * 2
    + Number(showMemoryMetric.value)
    + Number(showNetworkMetric.value)
);
const isHardwarePrimary = computed(() =>
    isHardwareMonEnabled.value
    && visibleHardwareMetricCount.value > 0
    && !isMusicPrimary.value
);

function scheduleTrackTitleMeasure() {
    if (titleMeasureFrame !== null) {
        cancelAnimationFrame(titleMeasureFrame);
        titleMeasureFrame = null;
    }

    nextTick(() => {
        titleMeasureFrame = requestAnimationFrame(() => {
            titleMeasureFrame = null;
            const viewport = musicTitleViewportRef.value;
            const title = musicTitleTextRef.value;
            if (!viewport || !title || !isMusicPrimary.value || !showInfo.value) {
                isTrackTitleOverflowing.value = false;
                return;
            }

            isTrackTitleOverflowing.value = title.scrollWidth > viewport.clientWidth + 4;
        });
    });
}

watch([currentTrackInfo, hasDisplayLyricLine, showInfo, isMusicPrimary], scheduleTrackTitleMeasure, { flush: 'post' });
watch([displayLyricLine, hasDisplayLyricLine, showInfo, isMusicPrimary], scheduleLyricLineMeasure, { flush: 'post' });

const showMiniMusicBubble = computed(() => false);
const isSplitActivity = computed(() => false);
const isDefaultSpeedPrimary = computed(() => false);
const showFlowBorder = computed(() =>
    isGlowBorderEnabled.value
    && (isMsgActive.value || isMusicPrimary.value || isHardwarePrimary.value)
);
const reportedIslandVisible = ref(false);
const shouldDisplayIsland = computed(() =>
    isIslandEnabled.value && (isMsgActive.value || isMusicPrimary.value || isHardwarePrimary.value)
);

const reportIslandStatus = async (visible: boolean) => {
    reportedIslandVisible.value = visible;
    await emit('island-status-sync', { visible });
};

async function keepIslandWindowPresent() {
    if (!shouldDisplayIsland.value || isMenuOpen.value) return;

    const appWindow = getCurrentWindow();
    try {
        await appWindow.show();
        await appWindow.setAlwaysOnTop(true);
        await invoke('force_window_topmost');
    } catch (error) {
        console.error('恢复灵动岛置顶失败:', error);
    }
}

function scheduleIslandRecoveryBurst() {
    if (!shouldDisplayIsland.value || isMenuOpen.value) return;

    [80, 260, 620, 1100].forEach((delay) => {
        window.setTimeout(() => {
            keepIslandWindowPresent().catch(() => { });
        }, delay);
    });
}

async function syncIslandVisibility() {
    const shouldShow = shouldDisplayIsland.value;
    const appWindow = getCurrentWindow();

    if (shouldShow) {
        await appWindow.show();
        await appWindow.setAlwaysOnTop(true);
        await invoke('force_window_topmost').catch(() => { });
        if (!isIslandVisible.value) {
            isIslandVisible.value = true;
        }
        await reportIslandStatus(true);
        return;
    }

    if (isIslandVisible.value) {
        isIslandVisible.value = false;
    } else {
        await appWindow.hide();
        await reportIslandStatus(false);
    }
}
let hideControlsTimer: number | null = null;

// 启动倒计时隐藏控件
const startHideTimer = () => {
    stopHideTimer();
    hideControlsTimer = window.setTimeout(() => {
        showInfo.value = true;
    }, 800);
};

const stopHideTimer = () => {
    if (hideControlsTimer) {
        clearTimeout(hideControlsTimer);
        hideControlsTimer = null;
    }
};

// 定义一个用于强制刷新的 key
const musicBoxKey = ref(0);

// 鼠标进入整个音乐控制区域
const handleMusicBoxEnter = () => {
    stopHideTimer();     // 清除可能正在倒计时的隐藏任务
    showInfo.value = false; // 立刻切换为显示控制按钮
};

// 鼠标彻底离开整个音乐控制区域
const handleMusicBoxLeave = () => {
    startHideTimer();    // 离开后，再开启倒计时恢复为歌曲名称显示
};

let lastRx = 0;
let lastTx = 0;
let lastSpeedSampleAt = 0;
let speedTimer: number;
let lyricTimer: number | null = null;
let pingTimer: number | null = null;
let visibilityGuardTimer: number | null = null;
let systemStatusTimer: number | null = null;

// 防抖控制变量
let lowTrafficStartTime = Date.now();
const RED_DELAY_MS = 5000;

const formatSpeed = (bytes: number) => {
    const safeBytes = Number.isFinite(bytes) ? Math.max(0, bytes) : 0;
    if (safeBytes < 1024) return `${Math.round(safeBytes)} B/s`;
    if (safeBytes < 1024 * 1024) return `${(safeBytes / 1024).toFixed(1)} KB/s`;
    if (safeBytes < 1024 * 1024 * 1024) return `${(safeBytes / (1024 * 1024)).toFixed(1)} MB/s`;
    return `${(safeBytes / (1024 * 1024 * 1024)).toFixed(1)} GB/s`;
};

// 计算流量数字，并实时更新大流量状态
const fetchSpeedStats = async () => {
    try {
        const [currentRx, currentTx] = await invoke<[number, number]>('get_network_stats');
        const sampledAt = performance.now();
        if (lastRx !== 0) {
            const elapsedSeconds = Math.max(0.25, (sampledAt - lastSpeedSampleAt) / 1000);
            const rxDiff = Math.max(0, (currentRx - lastRx) / elapsedSeconds);
            const txDiff = Math.max(0, (currentTx - lastTx) / elapsedSeconds);

            downloadSpeed.value = formatSpeed(rxDiff);
            uploadSpeed.value = formatSpeed(txDiff);

            // 1MB = 1048576 字节
            const limit = 1024 * 1024;
            const currentDownloadHigh = rxDiff >= limit;
            const currentUploadHigh = txDiff >= limit;

            isHighDownload.value = currentDownloadHigh;
            isHighUpload.value = currentUploadHigh;

            // 维护低流量持续时间
            if (currentDownloadHigh || currentUploadHigh) {
                // 如果目前依然是大流量，重置计时器
                lowTrafficStartTime = Date.now();
            }
        }
        lastRx = currentRx;
        lastTx = currentTx;
        lastSpeedSampleAt = sampledAt;
    } catch (error) {
        console.error('流量获取失败:', error);
    }
};

const formatGpuMemory = (usedMb?: number | null, totalMb?: number | null) => {
    if (typeof usedMb !== 'number' || typeof totalMb !== 'number' || totalMb <= 0) {
        return '--/--';
    }
    const usedGb = usedMb / 1024;
    const totalGb = totalMb / 1024;
    return `${usedGb.toFixed(1)}/${totalGb.toFixed(1)}G`;
};

const formatSystemMemory = (usedBytes: number, totalBytes: number) => {
    if (!Number.isFinite(usedBytes) || !Number.isFinite(totalBytes) || totalBytes <= 0) {
        return '--/--';
    }
    const usedGb = usedBytes / 1024 / 1024 / 1024;
    const totalGb = totalBytes / 1024 / 1024 / 1024;
    return `${usedGb.toFixed(1)}/${totalGb.toFixed(1)}G`;
};

const formatFanSpeed = (rpm?: number | null, percent?: number | null) => {
    if (typeof rpm === 'number' && Number.isFinite(rpm)) {
        return `${Math.round(rpm)}RPM`;
    }
    if (typeof percent === 'number' && Number.isFinite(percent)) {
        return `${Math.round(percent)}%`;
    }
    return '--';
};

const syncHardwareStats = async () => {
    try {
        const stats = await invoke<HardwareStats>('get_hardware_stats');
        cpuUsage.value = Math.round(stats.cpuUsage) + '%';
        cpuTemp.value = typeof stats.cpuTemperature === 'number' ? `${Math.round(stats.cpuTemperature)}°C` : '--°C';
        cpuFan.value = formatFanSpeed(stats.cpuFanRpm);
        gpuUsage.value = typeof stats.gpuUsage === 'number' ? `${Math.round(stats.gpuUsage)}%` : '--';
        gpuTemp.value = typeof stats.gpuTemperature === 'number' ? `${Math.round(stats.gpuTemperature)}°C` : '--°C';
        gpuFan.value = formatFanSpeed(stats.gpuFanRpm, stats.gpuFanSpeedPercent);
        gpuMemoryUsage.value = typeof stats.gpuMemoryUsage === 'number' ? `${Math.round(stats.gpuMemoryUsage)}%` : '--';
        gpuMemory.value = formatGpuMemory(stats.gpuMemoryUsedMb, stats.gpuMemoryTotalMb);
        ramUsage.value = stats.totalMemory > 0 ? `${Math.round((stats.usedMemory / stats.totalMemory) * 100)}%` : '--';
        ramMemory.value = formatSystemMemory(stats.usedMemory, stats.totalMemory);
    } catch (err) {
        console.error('获取硬件信息失败:', err);
    }
};

// 通过真实延迟控制状态灯（加入大流量避让判断）
const checkNetworkLatency = async () => {
    try {
        const latency = await invoke<number>('get_network_latency');

        // 只要能拿到延迟数字，说明网络肯定是通的
        if (latency < 150) {
            networkStatus.value = 'good';      // 延迟优秀，绿色
        } else {
            networkStatus.value = 'warning';   // 延迟高/不稳定，黄色
        }
    } catch (error) {
        // 当Rust抛出超时异常时，说明网络可能断开连接

        // 1. 如果当前正处于大流量状态，绝不变红，降级显示为黄灯
        if (isHighDownload.value || isHighUpload.value) {
            networkStatus.value = 'warning';
            return;
        }

        // 2. 如果流量刚刚消失，判断距离大流量结束是否超过了设定的缓冲时间
        const timeSinceLowTraffic = Date.now() - lowTrafficStartTime;
        if (timeSinceLowTraffic < RED_DELAY_MS) {
            // 还在缓冲期内，判定为大流量带来的余波卡顿，依然保持黄灯
            networkStatus.value = 'warning';
        } else {
            // 已经下了好几秒都没流量了，结果还连不上，说明是真的断网了，变红！
            networkStatus.value = 'error';
        }
    }
};

// 调整窗口位置到正确位置
const adjustWindowPosition = async () => {
    try {
        const appWindow = getCurrentWindow();
        await new Promise((resolve) => setTimeout(resolve, 150));
        const monitor = await currentMonitor();

        if (monitor) {
            const scaleFactor = window.devicePixelRatio;

            const WINDOW_INIT_WIDTH = currentWidth.value;
            const WINDOW_INIT_HEIGHT = currentHeight.value; // 默认 42
            await appWindow.setSize(new PhysicalSize(
                getWindowPhysicalWidth(WINDOW_INIT_WIDTH, scaleFactor),
                getWindowPhysicalHeight(WINDOW_INIT_HEIGHT, scaleFactor)
            ));

            const monitorWidthPhysical = monitor.size.width;
            const monitorLeftPhysical = monitor.position.x;
            const monitorTopPhysical = monitor.position.y;

            // 2. 重新获取设定后的真实物理尺寸，用于精准居中
            const windowSize = await appWindow.innerSize();
            const windowWidthPhysical = windowSize.width;

            const x = monitorLeftPhysical + (monitorWidthPhysical - windowWidthPhysical) / 2;
            const y = monitorTopPhysical + (12 * scaleFactor);

            await appWindow.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));

            trackedPhysicalX = Math.round(x);
            trackedPhysicalY = Math.round(y);
        }
    } catch (error) {
        console.error('调整窗口位置失败:', error);
    } finally {
        try {
            await getCurrentWindow().show();
        } catch (e) {
            console.error(e);
        }
    }
};

// 核心动画实现：基于你的 AE 公式转化
const onEnter = (el: Element, done: () => void) => {
    const HTMLElement = el as HTMLElement;
    HTMLElement.style.transformOrigin = 'center top'; // 类似苹果灵动岛从顶部展开
    let start = performance.now();

    const freq = 2.0;
    const decay = 10.5; // 适度拉高阻力
    const duration = 600;

    const animate = (time: number) => {
        let t = (time - start) / 1000;
        let progress = (time - start) / duration;

        // 数学方程：1 - cos(2πft) * e^(-dt)
        let scale = 1 - Math.cos(freq * t * 2 * Math.PI) * Math.exp(-decay * t);
        let opacity = Math.min(1, progress * 4); // 快速淡入

        HTMLElement.style.transform = `scale(${scale})`;
        HTMLElement.style.opacity = opacity.toString();

        if (progress < 1) {
            requestAnimationFrame(animate);
        } else {
            // 重置为最终干净的状态
            HTMLElement.style.transform = `scale(1)`;
            HTMLElement.style.opacity = '1';
            done();
        }
    };
    requestAnimationFrame(animate);
};

const onLeave = (el: Element, done: () => void) => {
    const HTMLElement = el as HTMLElement;
    HTMLElement.style.transformOrigin = 'center top';
    let start = performance.now();

    const duration = 300; // 收起动画通常更干脆、更快

    const animate = (time: number) => {
        let progress = (time - start) / duration;

        // 离开动画：快速平滑回缩
        // 使用 easing 曲线或简化的衰减
        let scale = 1 - Math.pow(progress, 3); // 快速内收
        let opacity = 1 - progress * 1.5;

        HTMLElement.style.transform = `scale(${Math.max(0, scale)})`;
        HTMLElement.style.opacity = Math.max(0, opacity).toString();

        if (progress < 1) {
            requestAnimationFrame(animate);
        } else {
            done();
            // 等待 DOM 动画播放完成后再隐藏窗口
            getCurrentWindow().hide().catch(console.error);
            reportIslandStatus(false).catch(console.error);
        }
    };
    requestAnimationFrame(animate);
};

const handleMouseDown = async (event: MouseEvent) => {
    // 【新增的核心锁定逻辑】：如果开启了置于任务栏，直接拦截，禁止任何拖拽！
    if (isPinnedToTaskbar.value) return;

    // 如果点击的是按钮或按钮内部的 SVG 图标，直接返回，不触发拖拽
    if ((event.target as HTMLElement).closest('.ctl-btn, .island-menu-button')) return;

    const target = event.target as HTMLElement;

    // 如果点击的是按钮或者消息框，直接返回，不触发窗口拖拽
    if (target.closest('.ctl-btn') || target.closest('.msg-box') || target.closest('.island-menu-button')) {
        return;
    }

    // 只有按鼠标左键时才触发窗口拖拽，把右键留给自定义菜单
    if (event.button === 0) {
        try {
            await getCurrentWindow().startDragging();
        } catch (error) {
            console.error('拖拽失败:', error);
        }
    }
};

function formatDiagnosticMs(value: number | null) {
    if (value === null || !Number.isFinite(value)) return '--:--';
    const totalSeconds = Math.max(0, Math.floor(value / 1000));
    const minutes = Math.floor(totalSeconds / 60).toString().padStart(2, '0');
    const seconds = (totalSeconds % 60).toString().padStart(2, '0');
    return `${minutes}:${seconds}`;
}

function buildMediaDiagnostics() {
    const kind = currentMediaKind.value === 'browser' ? 'Browser video' : 'Music';
    const status = hasMusicSession.value ? (isPlaying.value ? 'Playing' : 'Paused') : 'No media session';
    const title = currentSongTitle.value || 'None';
    const artist = currentSongArtist.value || 'None';
    const progress = `${formatDiagnosticMs(musicPositionMs.value)} / ${formatDiagnosticMs(musicDurationMs.value)}`;
    const lyricStatus = currentMediaKind.value === 'browser'
        ? 'Skipped for browser video'
        : (lyricsSource.value || (lyricLines.value.length > 0 ? 'Loaded' : 'No match'));
    const colorStatus = coverUrl.value ? 'From current cover/thumbnail' : 'Default gradient';

    return [
        'Media Session Diagnostics',
        '',
        `Status: ${status}`,
        `Type: ${kind}`,
        `Source: ${currentMusicSourceAppId.value || 'None'}`,
        `Title: ${title}`,
        `Artist: ${artist}`,
        `Cover source: ${currentCoverSource.value}`,
        `Color source: ${colorStatus}`,
        `Lyric source: ${lyricStatus}`,
        `Progress source: ${currentProgressSource.value}`,
        `Progress: ${progress}`,
        `Lyric offset: ${lyricOffsetMs.value}ms (positive = earlier)`,
    ].join('\n');
}

const showIslandMenu = async (position: LogicalPosition) => {
    // 重置位置
    const resetPositionItem = await MenuItem.new({
        text: isPinnedToTaskbar.value ? '重置位置 (已锁定)' : '重置位置',
        id: 'reset_position',
        enabled: !isPinnedToTaskbar.value, // 核心逻辑：开启置于任务栏时，禁用此按钮
        action: () => {
            adjustWindowPosition().catch(console.error);
        }
    });

    // 打开设置
    const openSettingsItem = await MenuItem.new({
        text: '打开设置',
        id: 'open_settings',
        action: async () => {
            // 发送一个信号给主控制台
            await emit('open-settings-panel');
        }
    });

    // 切换流光边框
    const mediaDiagnosticsItem = await MenuItem.new({
        text: '媒体诊断',
        id: 'media_diagnostics',
        action: () => {
            window.alert(buildMediaDiagnostics());
        }
    });

    const toggleGlowBorderItem = await MenuItem.new({
        text: isGlowBorderEnabled.value ? '关闭流光边框' : '开启流光边框',
        id: 'toggle_glow_border',
        enabled: true, // 改为 true，让你随时都可以点
        action: () => {
            isGlowBorderEnabled.value = !isGlowBorderEnabled.value;
            // 新增一行：把你切换后的状态存到本地电脑里
            localStorage.setItem('nsd_glow_border', String(isGlowBorderEnabled.value));
        }
    });

    // 关闭灵动岛
    const closeItem = await MenuItem.new({
        text: '关闭',
        id: 'close',
        action: () => {
            isIslandEnabled.value = false;
            localStorage.setItem('nsd_island_enabled', 'false');
            isIslandVisible.value = false;
        }
    });

    // 3. 创建菜单并按顺序追加进去
    const menu = await Menu.new();
    await menu.append(openSettingsItem);
    await menu.append(mediaDiagnosticsItem);
    await menu.append(toggleGlowBorderItem);
    await menu.append(resetPositionItem);
    await menu.append(closeItem);

    // 4. 弹出菜单
    try {
        isMenuOpen.value = true; // 👈 弹出前，告诉系统菜单打开了
        await menu.popup(position);
    } catch (error) {
        console.error('菜单弹出失败:', error);
    } finally {
        isMenuOpen.value = false; // 👈 无论用户是点击了菜单，还是点空白处取消了，都会瞬间恢复置顶状态
    }
};

const handleRightClick = async (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation(); // 阻止冒泡

    // 使用客户端坐标转逻辑坐标（避免无边框裁剪带来的漂移）
    await showIslandMenu(new LogicalPosition(event.clientX, event.clientY));
};

const handleMenuButtonClick = async (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    await showIslandMenu(new LogicalPosition(
        Math.round(rect.right - 168),
        Math.round(rect.bottom + 6)
    ));
};

const onInnerEnter = (el: Element, done: () => void) => {
    const htmlEl = el as HTMLElement;
    let start = performance.now();

    // 统一使用简单的渐变淡入 (200毫秒)
    const duration = 200;
    htmlEl.style.transformOrigin = 'center';
    htmlEl.style.opacity = '0';
    htmlEl.style.transform = 'none'; // 确保没有位移

    const animate = (time: number) => {
        let progress = (time - start) / duration;
        htmlEl.style.opacity = Math.min(1, progress).toString();

        if (progress < 1) {
            requestAnimationFrame(animate);
        } else {
            htmlEl.style.opacity = '1';
            done();

            // 只有音乐控制器需要在动画结束后开启“隐藏控件”的倒计时
            if (htmlEl.classList.contains('music-ctl-box')) {
                startHideTimer();
            }
        }
    };
    requestAnimationFrame(animate);
};

const onInnerLeave = (el: Element, done: () => void) => {
    const htmlEl = el as HTMLElement;
    let start = performance.now();
    const duration = 150;

    const animate = (time: number) => {
        let progress = (time - start) / duration;
        let opacity = 1 - progress;

        htmlEl.style.opacity = Math.max(0, opacity).toString();

        if (progress < 1) {
            requestAnimationFrame(animate);
        } else {
            done();
        }
    };
    requestAnimationFrame(animate);
};

const COMPACT_WIDTH = 300;
const COMPACT_HEIGHT = 42;
const MESSAGE_WIDTH = 330;
const MESSAGE_HEIGHT = COMPACT_HEIGHT;
const DEFAULT_DASHBOARD_WIDTH = 820;
const HARDWARE_MAX_WIDTH = 920;
const HARDWARE_LAYOUT_INSET = 56;
const HARDWARE_ITEM_GAP = 10;
const SPLIT_ACTIVITY_WIDTH = 970;
const hardwareMetricWidths = computed(() => {
    const metricWidths: number[] = [];

    if (showCpuMetric.value) metricWidths.push(168);
    if (showGpuMetric.value) metricWidths.push(168, 136);
    if (showMemoryMetric.value) metricWidths.push(148);
    if (showNetworkMetric.value) metricWidths.push(196);

    return metricWidths;
});
const hardwareGridStyle = computed<CSSProperties>(() => ({
    gridTemplateColumns: hardwareMetricWidths.value.map((width) => `${width}px`).join(' '),
}));
const hardwareTargetWidth = computed(() => {
    const metricWidths = hardwareMetricWidths.value;

    const contentWidth = metricWidths.reduce((total, width) => total + width, 0);
    const gapWidth = Math.max(0, metricWidths.length - 1) * HARDWARE_ITEM_GAP;

    return Math.min(
        HARDWARE_MAX_WIDTH,
        Math.max(COMPACT_WIDTH, HARDWARE_LAYOUT_INSET + contentWidth + gapWidth),
    );
});
const WINDOW_SAFE_PADDING_X = 28;

const getWindowLogicalWidth = (islandWidth: number) => islandWidth + WINDOW_SAFE_PADDING_X * 2;
const getWindowPhysicalWidth = (islandWidth: number, scaleFactor: number) =>
    Math.ceil(getWindowLogicalWidth(islandWidth) * scaleFactor);
const getWindowPhysicalHeight = (islandHeight: number, scaleFactor: number) =>
    Math.ceil(islandHeight * scaleFactor);

// 1. 新增：控制 DOM 真正的高宽变量与消息数据
const currentWidth = ref(isHardwarePrimary.value ? hardwareTargetWidth.value : COMPACT_WIDTH);
const currentHeight = ref(COMPACT_HEIGHT);
const isMsgActive = ref(false);
const msgTitle = ref('');
const msgBody = ref('');
const msgAumid = ref('');
type SystemStatusPayload = {
    hasBattery: boolean;
    isCharging: boolean;
    isOnBattery: boolean;
    batteryPercent?: number | null;
    volumePercent?: number | null;
};
type MessageIslandPayload = {
    appName: string;
    title: string;
    body?: string;
    aumid?: string;
    iconName?: string;
    durationMs?: number;
};
let lastSystemStatus: SystemStatusPayload | null = null;
let pendingVolumePercent: number | null = null;
let volumeNotifyTimer: number | null = null;

// 👇把里面的 app_name 改回 appName
const handleMsgClick = async () => {
    if (msgAumid.value || msgTitle.value) {
        try {
            // 听 Tauri 的话，这里必须用驼峰命名的 appName
            await invoke('open_app_by_aumid', {
                aumid: msgAumid.value,
                appName: msgTitle.value
            });

            isMsgActive.value = false;
            if ((window as any).msgTimer) clearTimeout((window as any).msgTimer);
        } catch (err) {
            console.error('打开程序失败:', err);
        }
    }
};

// 同步追踪窗口位置（物理像素），动画中直接读取，无需任何 async
let trackedPhysicalX = 0;
let trackedPhysicalY = 0;
let islandSizeAnimationFrame: number | null = null;

const easeInOutCubic = (value: number) =>
    value < 0.5
        ? 4 * value * value * value
        : 1 - Math.pow(-2 * value + 2, 3) / 2;

// 灵动岛核心代码！（优化性能版）
const animateIslandSize = (targetWidth: number, targetHeight: number) => {
    const startWidth = currentWidth.value;
    const startHeight = currentHeight.value;

    // 如果大小没变直接拦截，拒绝无意义的性能损耗
    if (Math.abs(startWidth - targetWidth) < 0.5 && Math.abs(startHeight - targetHeight) < 0.5) return;

    if (islandSizeAnimationFrame !== null) {
        cancelAnimationFrame(islandSizeAnimationFrame);
        islandSizeAnimationFrame = null;
    }

    const dpr = window.devicePixelRatio;
    const centerPhysicalX = trackedPhysicalX + (getWindowLogicalWidth(startWidth) * dpr) / 2;
    const originPhysicalY = trackedPhysicalY;

    const start = performance.now();
    const isShrinking = targetWidth < startWidth || targetHeight < startHeight;
    const duration = isShrinking ? 380 : 460;

    let lastIpcTime = 0; // 用于节流控制 Tauri 底层通讯频率

    const run = (time: number) => {
        const progress = clamp((time - start) / duration, 0, 1);
        const eased = easeInOutCubic(progress);

        const currentW = startWidth + (targetWidth - startWidth) * eased;
        const currentH = startHeight + (targetHeight - startHeight) * eased;

        // 1. 纯前端内存与 DOM 渲染，绝不阻塞，随便跑满 144Hz/240Hz
        currentWidth.value = currentW;
        currentHeight.value = currentH;

        // 2. 使用后端原子化窗口调整，避免坐标和尺寸分开更新造成左右抖动
        if (time - lastIpcTime > 16) {
            const currentLeftX = Math.round(centerPhysicalX - (getWindowLogicalWidth(currentW) * dpr) / 2);
            invoke('set_window_bounds', {
                x: currentLeftX,
                y: originPhysicalY,
                width: getWindowPhysicalWidth(currentW, dpr),
                height: getWindowPhysicalHeight(currentH, dpr),
            }).catch(() => { });

            lastIpcTime = time;
        }

        if (progress < 1) {
            islandSizeAnimationFrame = requestAnimationFrame(run);
        } else {
            // 动画结束，数值兜底
            currentWidth.value = targetWidth;
            currentHeight.value = targetHeight;
            trackedPhysicalX = Math.round(centerPhysicalX - (getWindowLogicalWidth(targetWidth) * dpr) / 2);

            // 进行最后一次严丝合缝的最终尺寸坐标锁定
            invoke('set_window_bounds', {
                x: trackedPhysicalX,
                y: originPhysicalY,
                width: getWindowPhysicalWidth(targetWidth, dpr),
                height: getWindowPhysicalHeight(targetHeight, dpr),
            }).catch(() => { });
            islandSizeAnimationFrame = null;
        }
    };

    islandSizeAnimationFrame = requestAnimationFrame(run);
};

const syncActivitySize = () => {
    if (isMsgActive.value) return;

    const targetWidth = isHardwarePrimary.value
        ? (isSplitActivity.value ? SPLIT_ACTIVITY_WIDTH : hardwareTargetWidth.value)
        : (isDefaultSpeedPrimary.value ? DEFAULT_DASHBOARD_WIDTH : COMPACT_WIDTH);

    animateIslandSize(targetWidth, COMPACT_HEIGHT);
};

watch([isHardwarePrimary, hardwareTargetWidth, showMiniMusicBubble, isMusicPrimary, isDefaultSpeedPrimary, isMsgActive], syncActivitySize);

// 引入你的默认图标作为兜底
import defaultLogo from '../assets/flowisland-logo.svg';
const currentMsgIcon = ref(defaultLogo);

// 极简版图标映射器 (你可以随时去 iconfont 找喜欢的图标放进 assets)
const getAppIcon = (appName: string) => {
    const name = appName.toLowerCase();

    if (name.includes('qq')) {
        // 使用 new URL 让 Vite 知道你要引入这个资源
        return new URL('../assets/qq.png', import.meta.url).href;
    }
    if (name.includes('钉钉') || name.includes('dingtalk')) {
        return new URL('../assets/dingtalk.png', import.meta.url).href;
    }
    if (name.includes('mail') || name.includes('邮件')) {
        return new URL('../assets/mail.png', import.meta.url).href;
    }
    if (name.includes('wechat') || name.includes('微信')) {
        return new URL('../assets/wechat.png', import.meta.url).href;
    }

    return defaultLogo;
};

const showMessageIsland = async ({
    appName,
    title,
    body = '',
    aumid = '',
    iconName,
    durationMs = 4200,
}: MessageIslandPayload) => {
    msgTitle.value = appName;
    msgAumid.value = aumid;
    msgBody.value = body ? `${title}: ${body}` : title;
    currentMsgIcon.value = getAppIcon(iconName || appName);

    if (!isMsgActive.value) {
        isMsgActive.value = true;
        await syncIslandVisibility();
        if (!isPinnedToTaskbar.value) {
            animateIslandSize(MESSAGE_WIDTH, MESSAGE_HEIGHT);
        }
    }

    if ((window as any).msgTimer) clearTimeout((window as any).msgTimer);
    (window as any).msgTimer = setTimeout(() => {
        isMsgActive.value = false;
        syncIslandVisibility().catch(console.error);
    }, durationMs);
};

const scheduleVolumeMessage = (percent: number) => {
    pendingVolumePercent = percent;
    if (volumeNotifyTimer !== null) clearTimeout(volumeNotifyTimer);
    volumeNotifyTimer = window.setTimeout(() => {
        if (pendingVolumePercent === null) return;
        showMessageIsland({
            appName: '系统音量',
            title: `当前音量 ${pendingVolumePercent}%`,
            iconName: 'system',
            durationMs: 2600,
        }).catch(console.error);
        pendingVolumePercent = null;
        volumeNotifyTimer = null;
    }, 80);
};

const syncSystemStatusEvents = async () => {
    const status = await invoke<SystemStatusPayload>('fetch_system_status');
    const previous = lastSystemStatus;
    lastSystemStatus = status;
    if (!previous) return;

    if (status.hasBattery) {
        const percentText = status.batteryPercent != null ? `，电量 ${status.batteryPercent}%` : '';
        if (!previous.isCharging && status.isCharging) {
            await showMessageIsland({
                appName: '电源已接入',
                title: `笔记本正在充电${percentText}`,
                iconName: 'system',
            });
        } else if (!previous.isOnBattery && status.isOnBattery) {
            await showMessageIsland({
                appName: '正在使用电池',
                title: `已切换为电池供电${percentText}`,
                iconName: 'system',
            });
        }
    }

    const currentVolume = typeof status.volumePercent === 'number' ? Math.round(status.volumePercent) : null;
    const previousVolume = typeof previous.volumePercent === 'number' ? Math.round(previous.volumePercent) : null;
    if (currentVolume !== null && previousVolume !== null && Math.abs(currentVolume - previousVolume) >= 1) {
        scheduleVolumeMessage(currentVolume);
    }
};

onMounted(async () => {
    islandThemeMedia = window.matchMedia('(prefers-color-scheme: dark)');
    systemPrefersDark.value = islandThemeMedia.matches;
    islandThemeMedia.addEventListener('change', handleIslandSystemThemeUpdate);

    document.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    }, { capture: true }); // 使用捕获阶段，确保先于 Tauri 底层拦截
    window.addEventListener('blur', scheduleIslandRecoveryBurst);
    window.addEventListener('resize', scheduleTrackTitleMeasure);
    window.addEventListener('resize', scheduleLyricLineMeasure);
    document.addEventListener('visibilitychange', scheduleIslandRecoveryBurst);

    // 【修改】统一合并后的音乐控制器状态监听器
    await listen<{ enabled: boolean }>('control-music-ctl', (event) => {
        const isEnabled = event.payload.enabled;
        isMusicCtlEnabled.value = isEnabled;

        if (!isEnabled) {
            hasMusicSession.value = false;
            currentMusicSourceAppId.value = '';
            currentSongTitle.value = '';
            currentSongArtist.value = '';
            currentTrackInfo.value = '未在播放歌曲 - 音乐软件';
            isPlaying.value = false;
            coverUrl.value = '';
            musicPositionMs.value = null;
            musicDurationMs.value = null;
            clearLyrics();
            stopHideTimer();
            syncIslandVisibility().catch(console.error);
            return;
        }

        if (isEnabled) {
            // 👇 新增：判断是不是“首次”（本地有没有存过流光边框的数据）
            if (localStorage.getItem('nsd_glow_border') === null) {
                isGlowBorderEnabled.value = true; // 自动开启流光边框
                localStorage.setItem('nsd_glow_border', 'true'); // 存入记忆，以后就不算“首次”了
            }

            showInfo.value = false;
            musicBoxKey.value++;
            stopHideTimer();
            syncMusicStatus().catch(console.error);
        }
    });

    // 监听来自控制台的透明度同步指令
    await listen<{ opacity: number }>('control-island-opacity', (event) => {
        islandOpacity.value = event.payload.opacity;
    });

    // 监听来自控制台的主题同步指令
    await listen<{ theme: string }>('control-island-theme', (event) => {
        islandTheme.value = event.payload.theme;
    });

    await listen<{ enabled: boolean }>('control-lyrics-visibility', (event) => {
        isLyricsEnabled.value = event.payload.enabled;
        localStorage.setItem('nsd_show_lyrics', String(event.payload.enabled));
        scheduleTrackTitleMeasure();
        scheduleLyricLineMeasure();
    });

    await listen<{ cpu: boolean; gpu: boolean; memory: boolean; network: boolean }>('control-monitor-metrics', async (event) => {
        const { cpu, gpu, memory, network } = event.payload;
        showCpuMetric.value = cpu;
        showGpuMetric.value = gpu;
        showMemoryMetric.value = memory;
        showNetworkMetric.value = network;
        localStorage.setItem('nsd_show_cpu', String(cpu));
        localStorage.setItem('nsd_show_gpu', String(gpu));
        localStorage.setItem('nsd_show_memory', String(memory));
        localStorage.setItem('nsd_show_network', String(network));
        await syncIslandVisibility();
    });

    await listen<{ offsetMs: number }>('control-lyric-offset', (event) => {
        const nextOffset = clamp(Number(event.payload.offsetMs) || 0, -3000, 3000);
        lyricOffsetMs.value = nextOffset;
        localStorage.setItem('nsd_lyric_offset_ms', String(nextOffset));
        refreshLyricClock();
    });

    // 监听置于任务栏开关
    await listen<{ enabled: boolean }>('control-pin-taskbar', async (event) => {
        isPinnedToTaskbar.value = event.payload.enabled;
        if (isPinnedToTaskbar.value) {
            await snapToBottomLeft(); // 开启时：飞到左下角
        } else {
            await adjustWindowPosition(); // 关闭时：等同于点击“重置位置”，飞回顶部居中
        }
    });

    // 监听消息模式开关
    await listen<{ enabled: boolean }>('control-msg-mode', async (event) => {
        isMsgModeEnabled.value = event.payload.enabled;
        await syncIslandVisibility();
    });

    // 初始化位置追踪
    const appWindow = getCurrentWindow();
    try {
        const pos = await appWindow.innerPosition();
        trackedPhysicalX = pos.x;
        trackedPhysicalY = pos.y;
    } catch (e) { }

    // 窗口被拖动后自动同步位置
    appWindow.onMoved((event) => {
        trackedPhysicalX = event.payload.x;
        trackedPhysicalY = event.payload.y;
    }).catch(() => { });

    // 根据本地记录决定启动时出现在哪
    if (isPinnedToTaskbar.value) {
        await snapToBottomLeft();
    } else {
        await adjustWindowPosition();
    }

    // 监听来自控制台的系统硬件监控开关
    await listen<{ enabled: boolean }>('control-hardware-mon', async (event) => {
        isHardwareMonEnabled.value = event.payload.enabled;
        localStorage.setItem('nsd_hardware_mon', String(event.payload.enabled));
        if (event.payload.enabled) {
            await Promise.all([fetchSpeedStats(), syncHardwareStats()]);
        }
        await syncIslandVisibility();
    });

    if (DASHBOARD_MODES_ENABLED && isHardwareMonEnabled.value) {
        fetchSpeedStats();
        checkNetworkLatency();
    }
    if (isMusicCtlEnabled.value) {
        await syncMusicStatus();
    } else {
        await syncIslandVisibility();
    }
    if (DASHBOARD_MODES_ENABLED && (isHardwareMonEnabled.value || isDefaultSpeedPrimary.value)) {
        await syncHardwareStats();
    }

    if (localStorage.getItem('nsd_msg_notify') !== 'false') {
        await syncSystemStatusEvents().catch(console.error);
    }
    systemStatusTimer = window.setInterval(() => {
        if (localStorage.getItem('nsd_msg_notify') !== 'false') {
            syncSystemStatusEvents().catch(console.error);
        }
    }, 200);

    // 在你原有的每秒刷新定时器中，顺带执行音乐同步
    speedTimer = setInterval(async () => {
        // Windows 任务栏/显示桌面有时会改透明置顶窗口的层级，这里温和拉回。
        if (isIslandVisible.value && shouldDisplayIsland.value && !isMenuOpen.value) {
            keepIslandWindowPresent().catch(() => { });
        }

        if (DASHBOARD_MODES_ENABLED && isHardwareMonEnabled.value) {
            fetchSpeedStats();
        }
        if (isMusicCtlEnabled.value) {
            await syncMusicStatus(); // 当音乐控制器启用时，每秒顺带检查音乐会话
        } else {
            await syncIslandVisibility();
        }

        // 每秒实时拉取系统硬件状态
        if (DASHBOARD_MODES_ENABLED && (isHardwareMonEnabled.value || isHardwarePrimary.value || isDefaultSpeedPrimary.value)) {
            await syncHardwareStats();
        }

        // 定时轮询系统通知状态
        const enabled = localStorage.getItem('nsd_msg_notify') !== 'false';
        if (enabled) {
            try {
                // 注意这里类型变了，接收任意对象
                const res = await invoke<any>('fetch_latest_notification');
                if (res) {
                    // res 结构：{ app_name: string, title: string, body: string }

                    // 1. 标题直接显示程序名
                    msgTitle.value = res.app_name;
                    msgAumid.value = res.aumid;

                    // 2. 内容拼接原来的 [标题: 内容]
                    if (res.body) {
                        msgBody.value = `${res.title}: ${res.body}`;
                    } else {
                        msgBody.value = res.title;
                    }

                    // 3. 动态替换头像
                    currentMsgIcon.value = getAppIcon(res.app_name);

                    if (!isMsgActive.value) {
                        isMsgActive.value = true;
                        await syncIslandVisibility();

                        // 👇 拦截：如果没有锁定在任务栏，才允许灵动岛变大
                        if (!isPinnedToTaskbar.value) {
                            animateIslandSize(MESSAGE_WIDTH, MESSAGE_HEIGHT);
                        }
                    }

                    if ((window as any).msgTimer) clearTimeout((window as any).msgTimer);
                    (window as any).msgTimer = setTimeout(() => {
                        isMsgActive.value = false;
                        syncIslandVisibility().catch(console.error);
                    }, 5000);
                }
            } catch (err) {
                console.error(err);
            }
        }
    }, 2000) as unknown as number;

    visibilityGuardTimer = setInterval(() => {
        if (isIslandVisible.value && shouldDisplayIsland.value && !isMenuOpen.value) {
            keepIslandWindowPresent().catch(() => { });
        }
    }, 850) as unknown as number;

    lyricTimer = setInterval(() => {
        if (isMusicPrimary.value && !isMsgActive.value && lyricLines.value.length > 0) {
            refreshLyricClock();
        }
    }, 250) as unknown as number;

    // 音乐-only 模式不再启用默认看板的 Ping 指示灯。
    if (DASHBOARD_MODES_ENABLED) {
        pingTimer = setInterval(() => {
            if (isHardwareMonEnabled.value) checkNetworkLatency();
        }, 5500) as unknown as number;
    }

    // 监听控制台发来的显隐调度指令
    await listen<{ show: boolean }>('control-island-visibility', async (event) => {
        isIslandEnabled.value = event.payload.show;
        localStorage.setItem('nsd_island_enabled', String(event.payload.show));

        if (event.payload.show) {
            if (isMusicCtlEnabled.value) {
                await syncMusicStatus();
            } else {
                await syncIslandVisibility();
            }
        } else {
            // 控制台关闭指令 -> 触发常规离开动画
            isIslandVisible.value = false;
            await reportIslandStatus(false);
        }
    });

    startHideTimer();
});

onUnmounted(() => {
    clearInterval(speedTimer);
    if (lyricTimer !== null) clearInterval(lyricTimer);
    if (pingTimer !== null) clearInterval(pingTimer);
    if (visibilityGuardTimer !== null) clearInterval(visibilityGuardTimer);
    if (systemStatusTimer !== null) clearInterval(systemStatusTimer);
    if (volumeNotifyTimer !== null) clearTimeout(volumeNotifyTimer);
    if (paletteTransitionFrame !== null) cancelAnimationFrame(paletteTransitionFrame);
    if (titleMeasureFrame !== null) cancelAnimationFrame(titleMeasureFrame);
    if (lyricMeasureFrame !== null) cancelAnimationFrame(lyricMeasureFrame);
    if (islandSizeAnimationFrame !== null) cancelAnimationFrame(islandSizeAnimationFrame);
    window.removeEventListener('blur', scheduleIslandRecoveryBurst);
    window.removeEventListener('resize', scheduleTrackTitleMeasure);
    window.removeEventListener('resize', scheduleLyricLineMeasure);
    document.removeEventListener('visibilitychange', scheduleIslandRecoveryBurst);
    islandThemeMedia?.removeEventListener('change', handleIslandSystemThemeUpdate);
    stopHideTimer();
});
</script>

<style scoped>
*,
*::before,
*::after {
    box-sizing: border-box;
    border: none !important;
    outline: none !important;
}

:root {
    -webkit-app-region: drag;
}

:global(html),
:global(body) {
    background-color: transparent !important;
    background: transparent !important;
    overflow: hidden;
    margin: 0;
    padding: 0;
    border: none !important;
}

/* 1. 外层包裹层：负责裁切多余的流光，并提供呼吸扩散的高斯模糊效果 */
.island-container {
    /* 移除 position: absolute; top: 0; */
    margin: 0 auto;
    /* 让它在窗口内水平居中 */
    border-radius: 100px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    user-select: none;
    -webkit-user-select: none;
    overflow: hidden;
    background: transparent;
    transition: background 0.4s ease, gap 0.35s cubic-bezier(0.2, 0.9, 0.18, 1);
    box-sizing: border-box;
}

.island-container.is-split-mode {
    justify-content: flex-start;
    gap: 10px;
    padding: 0;
    overflow: visible;
    background: transparent !important;
}

.island-container.is-split-mode .split-music-bubble {
    animation: split-bubble-pop 0.52s cubic-bezier(0.18, 1.18, 0.22, 1) both;
}

.island-container.is-split-mode .island-core-content {
    animation: split-pill-reveal 0.48s cubic-bezier(0.18, 1.05, 0.2, 1) both;
}

/* 2. 隐藏在底层的巨大旋转渐变层 */
.rainbow-border-glow {
    position: absolute;
    width: 620px;
    height: 620px;

    /* 用大面积雾化渐变覆盖胶囊，避免长条模式露底 */
    top: calc(50% - 310px);
    left: calc(50% - 310px);

    z-index: 1;

    /* 重新绘制的完美对称环形渐变，清透不发脏 */
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='500' height='500'%3E%3Cdefs%3E%3Cfilter id='b' x='-50%25' y='-50%25' width='200%25' height='200%25'%3E%3CfeGaussianBlur in='SourceGraphic' stdDeviation='60'/%3E%3C/filter%3E%3C/defs%3E%3Cg filter='url(%23b)'%3E%3Ccircle cx='250' cy='90' r='150' fill='%23ff3b30'/%3E%3Ccircle cx='390' cy='170' r='150' fill='%23ff9500'/%3E%3Ccircle cx='390' cy='330' r='150' fill='%234cd964'/%3E%3Ccircle cx='250' cy='410' r='150' fill='%23007aff'/%3E%3Ccircle cx='110' cy='330' r='150' fill='%235856d6'/%3E%3Ccircle cx='110' cy='170' r='150' fill='%23ff2d55'/%3E%3C/g%3E%3C/svg%3E");
    background-size: cover;

    /* 10秒一圈刚刚好，柔和且不怎么吃 GPU */
    animation: rainbow-rotate 10s linear infinite;
    filter: blur(28px) saturate(1.45) brightness(1.08);
    transform-origin: center;
    will-change: transform, filter;
}

.island-container.is-default-dashboard .rainbow-border-glow,
.island-container.is-hardware-dashboard .rainbow-border-glow {
    width: calc(100% + 160px);
    height: 180px;
    top: calc(50% - 90px);
    left: -80px;
    border-radius: 50%;
    background-size: 180% 100%;
    animation: wide-island-glow-flow 10s ease-in-out infinite alternate;
    filter: blur(28px) saturate(1.45) brightness(1.08);
    transform: translateZ(0) scale(1.04);
    transform-origin: center;
    will-change: transform, filter;
}

@keyframes wide-island-glow-flow {
    from {
        background-position: 0% 50%;
        transform: translateZ(0) scale(1.04);
    }

    to {
        background-position: 100% 50%;
        transform: translateZ(0) scale(1.08);
    }
}

/* 3. 核心遮罩内容块：挡在旋转渐变层的上方 */
.island-core-content {
    position: relative;
    z-index: 2;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    border-radius: 98px;
    transform: translateZ(0);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px;
    overflow: hidden;
    transition:
        border-radius 0.35s ease,
        box-shadow 0.35s ease,
        transform 0.35s cubic-bezier(0.2, 0.9, 0.18, 1),
        opacity 0.25s ease;
}

.island-container.is-split-mode .island-core-content {
    flex: 1;
    width: auto;
    border: 2px solid transparent !important;
    border-radius: 100px;
    padding: 0 16px;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12);
}

/* 4. 顺时针匀速旋转 */
@keyframes rainbow-rotate {
    from {
        transform: rotate(0deg) scale(1.08);
    }

    to {
        transform: rotate(360deg) scale(1.08);
    }
}

@keyframes split-bubble-pop {
    0% {
        opacity: 0;
        transform: translateX(116px) scale(0.64);
        filter: blur(2px);
    }

    58% {
        opacity: 1;
        transform: translateX(-3px) scale(1.08);
        filter: blur(0);
    }

    100% {
        opacity: 1;
        transform: translateX(0) scale(1);
        filter: blur(0);
    }
}

@keyframes split-pill-reveal {
    0% {
        opacity: 0;
        transform: translateX(-34px) scaleX(0.76);
    }

    62% {
        opacity: 1;
        transform: translateX(3px) scaleX(1.025);
    }

    100% {
        opacity: 1;
        transform: translateX(0) scaleX(1);
    }
}

[data-tauri-drag-region] {
    -webkit-app-region: drag;
    cursor: grab;
}

[data-tauri-drag-region]:active {
    cursor: grabbing;
}

.speed-box {
    display: flex;
    align-items: center;
    gap: 10px;
    box-sizing: border-box;
}

.speed-item {
    display: flex;
    align-items: center;
    gap: 4px;
    transform: translateY(-1px);
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
}

.label {
    font-size: 10px;
    color: currentColor;
    opacity: 0.4;
    font-weight: bold;
    padding: 2px 4px;
    border-radius: 4px;
    transition: all 0.3s ease;
}

/* 高流量时的 label 样式 */
.label.high-traffic {
    color: currentColor;
    opacity: 0.9;
    /* 文字稍微变亮，增加可读性 */
    background: rgba(255, 255, 255, 0.15);
    /* 浅白色半透明背景 */
}

.value {
    font-size: 12px;
    font-weight: bold;
    min-width: 52px;
    letter-spacing: 0;
}

.default-stats-box {
    justify-content: center;
    gap: 12px;
    padding: 0 12px;
    width: 100%;
    box-sizing: border-box;
    white-space: nowrap;
}

.hardware-stats-box {
    display: grid;
    grid-auto-flow: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 0 10px;
    overflow: hidden;
}

.hardware-stats-box .default-stat-item {
    width: 100%;
    min-width: 0;
}

.hardware-stats-box .default-stat-copy {
    width: 100%;
    min-width: 0;
}

.hardware-stats-box .default-stat-values {
    gap: 6px;
    font-size: 11px;
}

.hardware-stats-box .network-values {
    gap: 8px;
}

.default-stat-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 0 0 auto;
    min-width: 90px;
    height: 34px;
    color: currentColor;
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
}

.default-stat-compute {
    min-width: 150px;
}

.default-stat-fps {
    min-width: 72px;
}

.default-stat-item:not(:first-child)::before {
    content: '';
    position: absolute;
    left: -6px;
    top: 7px;
    width: 1px;
    height: 20px;
    background: currentColor;
    opacity: 0.28;
    border-radius: 999px;
}

.default-stat-memory {
    min-width: 128px;
}

.default-stat-network {
    min-width: 154px;
}

.default-stat-icon {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 26px;
    color: currentColor;
    opacity: 0.94;
    border-radius: 8px;
    background: color-mix(in srgb, currentColor 8%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, currentColor 18%, transparent);
}

.metric-icon-glyph {
    width: 15px;
    height: 15px;
    color: currentColor;
}

.nsd-mark {
    position: relative;
    display: block;
    width: 17px;
    height: 17px;
    color: currentColor;
}

.nsd-mark-cpu {
    border: 1px solid currentColor;
    border-radius: 5px;
    background:
        linear-gradient(currentColor, currentColor) center / 7px 7px no-repeat,
        radial-gradient(circle, currentColor 50%, transparent 54%) 1px 1px / 3px 3px no-repeat,
        radial-gradient(circle, currentColor 50%, transparent 54%) 13px 1px / 3px 3px no-repeat,
        radial-gradient(circle, currentColor 50%, transparent 54%) 1px 13px / 3px 3px no-repeat,
        radial-gradient(circle, currentColor 50%, transparent 54%) 13px 13px / 3px 3px no-repeat;
}

.nsd-mark-gpu::before {
    content: '';
    position: absolute;
    inset: 3px 2px;
    border-radius: 5px;
    background: currentColor;
    clip-path: polygon(8% 18%, 68% 18%, 92% 50%, 68% 82%, 8% 82%, 28% 50%);
}

.nsd-mark-gpu::after {
    content: '';
    position: absolute;
    left: 5px;
    top: 7px;
    width: 7px;
    height: 3px;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.45);
}

.nsd-mark-mem {
    border-radius: 4px;
    background:
        linear-gradient(90deg, currentColor 0 3px, transparent 3px 5px, currentColor 5px 8px, transparent 8px 10px, currentColor 10px 13px) center 5px / 13px 4px no-repeat,
        linear-gradient(currentColor, currentColor) center 12px / 14px 2px no-repeat;
}

.nsd-mark-mem::after {
    content: '';
    position: absolute;
    left: 2px;
    bottom: 1px;
    width: 13px;
    height: 2px;
    border-radius: 999px;
    background: currentColor;
    opacity: 0.55;
}

.nsd-mark-ram {
    border-radius: 4px;
    background:
        linear-gradient(currentColor, currentColor) 2px 3px / 13px 2px no-repeat,
        linear-gradient(currentColor, currentColor) 2px 12px / 13px 2px no-repeat,
        radial-gradient(circle, currentColor 55%, transparent 58%) 3px 7px / 3px 3px no-repeat,
        radial-gradient(circle, currentColor 55%, transparent 58%) 7px 7px / 3px 3px no-repeat,
        radial-gradient(circle, currentColor 55%, transparent 58%) 11px 7px / 3px 3px no-repeat;
}

.nsd-mark-network {
    background:
        radial-gradient(ellipse at center 12px, transparent 0 3px, currentColor 3px 4px, transparent 4px) center 0 / 17px 14px no-repeat,
        radial-gradient(ellipse at center 9px, transparent 0 6px, currentColor 6px 7px, transparent 7px) center 0 / 17px 14px no-repeat,
        radial-gradient(circle, currentColor 0 2px, transparent 2px) center 13px / 17px 4px no-repeat;
}

.nsd-mark-fps {
    border: 1px solid currentColor;
    border-radius: 50%;
}

.nsd-mark-fps::before {
    content: '';
    position: absolute;
    left: 3px;
    right: 3px;
    bottom: 3px;
    height: 8px;
    border-radius: 0 0 999px 999px;
    border: 1px solid currentColor;
    border-top: 0;
    opacity: 0.85;
}

.nsd-mark-fps::after {
    content: '';
    position: absolute;
    left: 8px;
    top: 4px;
    width: 1px;
    height: 7px;
    border-radius: 999px;
    background: currentColor;
    transform-origin: 50% 7px;
    transform: rotate(38deg);
}

.default-stat-copy {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1px;
    min-width: max-content;
}

.default-stat-title {
    font-size: 12px;
    line-height: 13px;
    font-weight: 650;
    opacity: 0.9;
}

.default-stat-values {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11.5px;
    line-height: 12px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    opacity: 0.96;
}

.default-stat-values span {
    flex: 0 0 auto;
}

.network-values {
    gap: 10px;
}

.network-values .high-traffic {
    color: #facc15;
}

.status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    transition: background-color 0.4s ease;
}

/* 修改后（去掉发光阴影，改为纯粹的扁平化圆点，干净利落） */
.good {
    background-color: #34C759;
}

.warning {
    background-color: #FFCC00;
}

.error {
    background-color: #FF3B30;
}

.music-wave {
    width: 22px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    flex: 0 0 22px;
    opacity: 0.72;
    margin-left: 6px;
    transform: translateX(-1px);
}

.island-menu-button {
    width: 22px;
    height: 22px;
    flex: 0 0 22px;
    margin-left: 2px;
    border-radius: 50%;
    display: none;
    align-items: center;
    justify-content: center;
    color: currentColor;
    background: rgba(255, 255, 255, 0.08);
    opacity: 0.72;
    cursor: pointer;
    -webkit-app-region: no-drag;
    transition: opacity 0.2s ease, background-color 0.2s ease, transform 0.2s ease;
}

.island-menu-button:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.16);
    transform: scale(1.04);
}

.island-menu-button svg {
    width: 14px;
    height: 14px;
    pointer-events: none;
}

.music-wave span {
    width: 2px;
    height: 5px;
    border-radius: 999px;
    background: linear-gradient(180deg, var(--music-wave-top, #ffbd78) 0%, var(--music-wave-bottom, #f06b3e) 100%);
    box-shadow: 0 0 6px var(--music-wave-glow, rgba(240, 107, 62, 0.35));
    transform-origin: center;
    transition: height 0.25s ease, opacity 0.25s ease, background 0.25s ease, box-shadow 0.25s ease;
}

.music-wave span:nth-child(2) {
    height: 8px;
}

.music-wave span:nth-child(3) {
    height: 6px;
}

.music-wave span:nth-child(4) {
    height: 3px;
    opacity: 0.65;
}

.music-wave.is-playing {
    opacity: 1;
}

.music-wave.is-playing span {
    animation: music-wave-pulse 0.86s ease-in-out infinite;
}

.music-wave.is-playing span:nth-child(1) {
    animation-delay: -0.34s;
}

.music-wave.is-playing span:nth-child(2) {
    animation-delay: -0.12s;
}

.music-wave.is-playing span:nth-child(3) {
    animation-delay: -0.48s;
}

.music-wave.is-playing span:nth-child(4) {
    animation-delay: -0.22s;
}

@keyframes music-wave-pulse {
    0%,
    100% {
        transform: scaleY(0.45);
    }

    42% {
        transform: scaleY(1.65);
    }

    68% {
        transform: scaleY(0.8);
    }
}

.split-music-bubble {
    position: relative;
    z-index: 3;
    width: 42px;
    height: 42px;
    flex: 0 0 42px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 5px;
    overflow: hidden;
    cursor: pointer;
    -webkit-app-region: no-drag;
    border: 2px solid transparent !important;
    background-origin: border-box;
    background-clip: padding-box, border-box;
    box-shadow:
        inset 0 0 0 1px rgba(255, 255, 255, 0.12),
        0 0 10px rgba(0, 0, 0, 0.18);
}

.split-music-bubble .music-mini-cover {
    border: 2px solid rgba(255, 255, 255, 0.45) !important;
    box-shadow: 0 0 6px rgba(255, 255, 255, 0.14);
}

.split-music-bubble:hover .music-mini-cover {
    filter: brightness(0.55);
    transform: scale(1.04);
}

.split-music-bubble:hover .music-mini-play {
    opacity: 1;
}

/* 让两个盒子脱离彼此的影响，在同一个包裹层内完美的“重叠”放置 */
.music-ctl-box,
.speed-box {
    position: absolute;
    /* 改为绝对定位，实现无缝平替 */
    left: 0;
    top: 0;
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
}

.music-ctl-box {
    justify-content: flex-start;
}

/* 核心改动：增加统一的内部绝对定位平替包裹层 */
.inner-wrapper {
    position: relative;
    z-index: 2;
    flex-grow: 1;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    align-items: center;
}

.album-cover {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    /* 变成纯圆球形 */
    box-sizing: unset !important;
    border: 2px solid rgba(255, 255, 255, 0.5) !important;
    /* 2px 的外环 */
    background: linear-gradient(135deg, #a8edea 0%, #fed6e3 100%);
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.250);
    transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    z-index: 2;
    transform: translateX(-5px);
    -webkit-app-region: no-drag;
    /* 确保层级比控制器高 */
}

/* 亮色模式下的外环颜色自动变暗 */
:deep(.island-container[style*="background-color: rgba(255, 255, 255"]) .album-cover {
    border-color: rgba(0, 0, 0, 0.15);
}

.album-cover.is-playing {
    transform: scale(1.08) translateX(-5px);
}

/* 封面内部绑定背景图的 div */
.cover-inner {
    width: 100%;
    height: 100%;
    background-position: center;
    background-repeat: no-repeat;
    background-size: cover;
    transition: background-image 0.3s ease;
    /* 切换封面时平滑淡入 */

    /* 👇 新增下面这两行 */
    animation: rotate 8s linear infinite;
    animation-play-state: paused;
    /* 默认让动画处于暂停状态 */
}

/* 正在播放时的旋转动画 */
.is-playing .cover-inner {
    /* 👇 把原来的 animation 替换成下面这行 */
    animation-play-state: running;
    /* 当有播放状态时，让动画跑起来 */
}

@keyframes rotate {
    from {
        transform: rotate(0deg);
    }

    to {
        transform: rotate(360deg);
    }
}

.music-controls {
    position: absolute;
    left: calc(50% + 26px);
    transform: translateX(-50%);
    /* 右侧波浪+菜单按钮占 52px，这里补偿一半，让播放/暂停按钮对齐整个胶囊中心 */
    display: flex;
    align-items: center;
    gap: 12px;
    /* 间距拉开一点更舒展 */
    z-index: 1;
}

.ctl-btn {
    background: transparent;
    /* 默认透明，无背景色 */
    border: none;
    color: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    /* 稍微加大内边距，让 hover 时的圆圈更好看 */
    border-radius: 50%;
    transition: background-color 0.2s ease, opacity 0.2s ease, transform 0.1s ease;
    outline: none;
    -webkit-app-region: no-drag;
}

/* 只有在 hover 的时候才出现背景色 */
.ctl-btn:hover {
    background-color: rgba(255, 255, 255, 0.15);
}

:deep(.island-container[style*="background-color: rgba(255, 255, 255"]) .ctl-btn:hover {
    background-color: rgba(0, 0, 0, 0.1);
}

.ctl-btn:active {
    opacity: 0.6;
    transform: scale(0.92);
    /* 加上按压时的微缩放反馈，手感更好 */
}

.ctl-btn svg {
    width: 16px;
    height: 16px;
    pointer-events: none;
}

/* 播放键稍微比切歌键大一点点，突出视觉中心 */
.play-btn svg {
    width: 20px;
    height: 20px;
}

/* 控件显隐淡入淡出动画过渡 */
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

/* 歌曲信息遮罩容器：单行歌名向右留出呼吸感，避免贴着封面 */
.music-info-mask-box {
    position: absolute;
    left: 42px;
    /* 封面直径 24px，加上外环和间距后让歌名视觉更居中 */
    right: 2px;
    /* 右侧波浪和菜单按钮在 inner-wrapper 外独立占位，这里尽量放大歌名范围 */
    height: 100%;
    display: flex;
    align-items: center;
    overflow: hidden;
    padding-left: 0;
    -webkit-app-region: no-drag;
    pointer-events: none;
    z-index: 4;
}

/* 歌曲文本基础样式 */
.music-info-text {
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    color: inherit;
}

.music-info-text.has-lyric {
    gap: 3px;
    justify-content: center;
    transform: translateY(-3px);
}

.music-title-viewport {
    max-width: 100%;
    overflow: hidden;
    transform: translateY(-2px);
}

.music-info-text.has-lyric .music-title-viewport {
    transform: translateY(0);
}

.music-info-text.is-marquee .music-title-viewport {
    width: 100%;
    mask-image: linear-gradient(to right, transparent 0, #000 14px, #000 calc(100% - 14px), transparent 100%);
    -webkit-mask-image: linear-gradient(to right, transparent 0, #000 14px, #000 calc(100% - 14px), transparent 100%);
}

.music-title-track {
    display: inline-flex;
    align-items: center;
    white-space: nowrap;
    max-width: 100%;
}

.music-info-text.is-marquee .music-title-track {
    gap: 48px;
    max-width: none;
    animation: title-marquee 9s linear infinite;
}

.music-title {
    display: inline-block;
    flex: 0 0 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    line-height: 1;
    font-weight: 700;
    opacity: 0.96;
}

.music-info-text.has-lyric .music-title {
    font-size: 10px;
    line-height: 10px;
    font-weight: 650;
    opacity: 0.68;
}

.music-info-text.is-marquee .music-title {
    overflow: visible;
    text-overflow: clip;
}

.music-lyric-line {
    width: 100%;
    max-width: 100%;
    overflow: hidden;
    text-align: center;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 11.5px;
    line-height: 12px;
    font-weight: 750;
    opacity: 0.98;
    transform: translateY(-1px);
}

.music-lyric-line.is-upcoming {
    opacity: 0.72;
}

.music-lyric-line.is-long {
    text-align: left;
}

.music-lyric-viewport {
    width: 100%;
    max-width: 100%;
    overflow: hidden;
}

.music-lyric-line.is-long .music-lyric-viewport {
    mask-image: linear-gradient(to right, transparent 0, #000 12px, #000 calc(100% - 12px), transparent 100%);
    -webkit-mask-image: linear-gradient(to right, transparent 0, #000 12px, #000 calc(100% - 12px), transparent 100%);
}

.music-lyric-track {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    white-space: nowrap;
}

.music-lyric-line.is-long .music-lyric-track {
    gap: 36px;
    max-width: none;
    animation: lyric-marquee 8.5s linear infinite;
}

.music-lyric-text {
    display: inline-block;
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.music-lyric-line.is-long .music-lyric-text {
    flex: 0 0 auto;
    overflow: visible;
    text-overflow: clip;
}

.lyric-fade-enter-active,
.lyric-fade-leave-active {
    transition: opacity 0.18s ease, transform 0.18s ease;
}

.lyric-fade-enter-from,
.lyric-fade-leave-to {
    opacity: 0;
    transform: translateY(3px);
}

@keyframes title-marquee {
    0%,
    18% {
        transform: translateX(0);
    }

    100% {
        transform: translateX(calc(-50% - 24px));
    }
}

/* 灵动岛消息通知样式 */
@keyframes lyric-marquee {
    0%,
    16% {
        transform: translateX(0);
    }

    78%,
    100% {
        transform: translateX(calc(-50% - 18px));
    }
}

.msg-box {
    position: absolute;
    left: 0;
    top: 0;
    right: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    padding: 0 4px 0 0;
    /* 右侧 45px 留给状态灯安全区域，左侧 16px 间距 */
    box-sizing: border-box;
    z-index: 10;
    gap: 8px;
    /* 图标与文本的间距 */
    -webkit-app-region: no-drag;
}

/* 预制消息图标/头像样式 */
.msg-avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    /* 默认圆形，可改为 8px 变成圆角矩形 */
    border: 2px solid rgba(255, 255, 255, 0.5) !important;
    background: linear-gradient(135deg, rgba(168, 237, 234, 0.9), rgba(254, 214, 227, 0.9));
    /* 渐变亮色背景 */
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    flex-shrink: 0;
    overflow: hidden;
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.22);
    transform: translateX(-5px);
}

.msg-avatar-img {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    object-fit: cover;
}

/* 文本靠左对齐包裹层 */
.msg-text-wrapper {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
    /* 核心：强制内部文本左对齐 */
    overflow: hidden;
    flex-grow: 1;
    min-width: 0;
    gap: 1px;
}

/* 调大后的标题样式 */
.msg-title {
    font-size: 12px;
    /* 从 12px 放大到 14px */
    font-weight: 700;
    line-height: 14px;
    opacity: 0.95;
    text-align: left;
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 调大后的内容样式 */
.msg-body {
    font-size: 10.5px;
    /* 从 11px 放大到 12.5px */
    line-height: 12px;
    font-weight: 650;
    opacity: 0.82;
    text-align: left;
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}


/* 系统硬件盒子样式 */
.msg-status-dot {
    width: 6px;
    height: 6px;
    flex: 0 0 6px;
    border-radius: 50%;
    background: #34c759;
    box-shadow: 0 0 8px rgba(52, 199, 89, 0.5);
    animation: msg-status-pulse 1.8s ease-in-out infinite;
}

@keyframes msg-status-pulse {
    0%,
    100% {
        opacity: 0.7;
        transform: scale(0.92);
    }

    50% {
        opacity: 1;
        transform: scale(1.08);
    }
}

.systemstate-box {
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    white-space: nowrap;
    overflow: hidden;
}

.music-mini-bubble {
    position: relative;
    width: 28px;
    height: 28px;
    flex: 0 0 28px;
    border-radius: 50%;
    overflow: hidden;
    cursor: pointer;
    -webkit-app-region: no-drag;
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.18);
}

.music-mini-cover {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: linear-gradient(135deg, #a8edea 0%, #fed6e3 100%);
    background-position: center;
    background-repeat: no-repeat;
    background-size: cover;
    animation: rotate 8s linear infinite;
    animation-play-state: paused;
    transition: filter 0.2s ease, transform 0.2s ease;
    will-change: transform;
}

.music-mini-cover.is-playing {
    animation-play-state: running;
}

.music-mini-bubble:hover .music-mini-cover {
    filter: brightness(0.55);
    transform: scale(1.04);
}

.music-mini-play {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border-radius: 50%;
    color: #fff;
    background: rgba(0, 0, 0, 0.18);
    opacity: 0;
    cursor: pointer;
    -webkit-app-region: no-drag;
    transition: opacity 0.2s ease, background-color 0.2s ease;
}

.music-mini-bubble:hover .music-mini-play {
    opacity: 1;
}

.music-mini-play:hover {
    background: rgba(0, 0, 0, 0.28);
}

.music-mini-play svg {
    width: 13px;
    height: 13px;
    pointer-events: none;
}

.hw-item {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 0 0 auto;
    margin-left: 0;
    transform: translateY(-1px);
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
    white-space: nowrap;
}

.hw-label {
    font-size: 10px;
    color: currentColor;
    opacity: 0.5;
    font-weight: bold;
    flex: 0 0 auto;
    white-space: nowrap;
}

.hw-divider {
    width: 1px;
    height: 16px;
    background: currentColor;
    opacity: 0.12;
    border-radius: 999px;
}

.hw-value {
    font-size: 13px;
    font-weight: bold;
    min-width: max-content;
    flex: 0 0 auto;
    white-space: nowrap;
    letter-spacing: 0;
    transition: color 0.3s ease;
    /* 👈 新增：让颜色变化时有 0.3 秒的平滑淡入淡出，不突兀 */
}

/* 👇 新增：当占用率达到 90% 及以上时触发的标准苹果亮红色 */
.hw-value.high-usage {
    color: #f06861 !important;
}

.hw-memory {
    font-size: 12px;
}

.hw-temp {
    color: #facc15;
}
</style>
