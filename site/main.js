let wasmModule = null;
let fontsReady = false;
let monacoReady = false;
let editorInstance = null;

const FONTS_TO_LOAD = [
    { name: 'JetBrains Mono', url: 'https://cdn.jsdelivr.net/gh/JetBrains/JetBrainsMono@v2.304/fonts/ttf/JetBrainsMono-Regular.ttf' },
    { name: 'Noto Sans SC', url: 'https://fonts.gstatic.com/s/notosanssc/v26/k3kXo84MPvpLmixcA63oeALhL4iP-Q8.otf' },
];

const THEME_NAMES = {
    echarts: 'ECharts 6',
    light: '浅色',
    dark: '深色',
    vintage: '复古',
    macarons: '马卡龙',
    infographic: '信息图',
    shine: '闪耀',
    roma: '罗马',
};

let currentSvg = null;
let currentPngBytes = null;

// ── Monaco Editor ──────────────────────────────────────────

require.config({
    paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.0/min/vs' },
});

require(['vs/editor/editor.main'], function () {
    monacoReady = true;
    monaco.editor.defineTheme('liecharts-dark', {
        base: 'vs-dark',
        inherit: true,
        rules: [
            { token: 'string.key.json', foreground: '#89ddff' },
            { token: 'string.value.json', foreground: '#c3e88d' },
            { token: 'number.json', foreground: '#f78c6c' },
            { token: 'keyword.json', foreground: '#c792ea' },
            { token: 'delimiter.json', foreground: '#89ddff' },
        ],
        colors: {
            'editor.background': '#1a1a2e',
            'editor.foreground': '#e0e0e0',
            'editorLineNumber.foreground': '#3c3c6e',
            'editorCursor.foreground': '#e94560',
            'editor.selectionBackground': '#0f346080',
            'editor.lineHighlightBackground': '#1e1e36',
            'editorIndentGuide.background': '#2a2a4e',
            'editorIndentGuide.activeBackground': '#3a3a5e',
            'editorBracketMatch.background': '#0f3460',
            'editorBracketMatch.border': '#e94560',
            'editorWidget.background': '#16213e',
            'editorWidget.border': '#0f3460',
            'minimap.background': '#1a1a2e',
        },
    });

    createEditor();
});

function createEditor() {
    const container = document.getElementById('monacoEditor');
    if (!container) return;

    editorInstance = monaco.editor.create(container, {
        value: '{\n    \n}',
        language: 'json',
        theme: 'liecharts-dark',
        fontSize: 13,
        lineNumbers: 'on',
        minimap: { enabled: true, showSlider: 'mouseover' },
        automaticLayout: true,
        scrollBeyondLastLine: false,
        tabSize: 2,
        formatOnPaste: true,
        renderWhitespace: 'selection',
        bracketPairColorization: { enabled: true },
        wordWrap: 'on',
        guides: { indentation: true, bracketPairs: true },
        padding: { top: 8 },
        smoothScrolling: true,
        cursorBlinking: 'smooth',
        cursorSmoothCaretAnimation: 'on',
    });

    editorInstance.onDidChangeModelContent(() => {
        clearMonacoMarkers();
        validateJson();
    });

    monaco.editor.setModelLanguage(editorInstance.getModel(), 'json');
}

function getEditorValue() {
    return editorInstance ? editorInstance.getValue() : '';
}

function setEditorValue(value) {
    if (editorInstance) {
        editorInstance.setValue(value);
    }
}

function formatEditor() {
    if (!editorInstance) return;
    editorInstance.getAction('editor.action.formatDocument').run();
}

function setMonacoMarkers(markers) {
    if (!monaco) return;
    monaco.editor.setModelMarkers(
        editorInstance.getModel(),
        'json',
        markers
    );
}

function clearMonacoMarkers() {
    if (!monaco || !editorInstance) return;
    monaco.editor.setModelMarkers(editorInstance.getModel(), 'json', []);
}

function validateJson() {
    const text = getEditorValue().trim();
    if (!text) return;

    try {
        JSON.parse(text);
        clearMonacoMarkers();
    } catch (e) {
        setMonacoMarkers([{
            severity: monaco.MarkerSeverity.Error,
            message: e.message,
            startLineNumber: 1,
            startColumn: 1,
            endLineNumber: 1,
            endColumn: 1,
        }]);
    }
}

// ── WASM ──────────────────────────────────────────────────

async function initWasm() {
    try {
        wasmModule = await import('./pkg/liecharts_site.js');
        await wasmModule.default();
        console.log('WASM module loaded successfully');
    } catch (err) {
        console.error('Failed to load WASM module:', err);
        showError('WASM 模块加载失败: ' + err.message);
        throw err;
    }
}

async function loadThemes() {
    if (!wasmModule) return;

    try {
        const themesJson = wasmModule.get_available_themes();
        const themes = JSON.parse(themesJson);
        const select = document.getElementById('themeSelect');
        select.innerHTML = '<option value="">默认主题</option>';

        themes.forEach(function (name) {
            const displayName = THEME_NAMES[name] || name;
            const opt = document.createElement('option');
            opt.value = name;
            opt.textContent = displayName;
            select.appendChild(opt);
        });

        console.log('Themes loaded:', themes.join(', '));
    } catch (err) {
        console.warn('Failed to load themes:', err);
    }
}

async function loadFonts() {
    const errorPanel = document.getElementById('errorPanel');
    const generateBtn = document.getElementById('generateBtn');

    let lastFontBytes = null;

    for (const font of FONTS_TO_LOAD) {
        try {
            console.log('Loading font: ' + font.name + '...');
            const response = await fetch(font.url);
            if (!response.ok) {
                throw new Error('HTTP ' + response.status + ': ' + response.statusText);
            }
            const arrayBuffer = await response.arrayBuffer();
            const bytes = new Uint8Array(arrayBuffer);

            lastFontBytes = { arrayBuffer: arrayBuffer, bytes: bytes };

            wasmModule.register_font_bytes(font.name, bytes);
            console.log('Font registered in WASM: ' + font.name + ' (' + (bytes.length / 1024).toFixed(1) + ' KB)');

            try {
                const fontFace = new FontFace(font.name, arrayBuffer);
                const loadedFont = await fontFace.load();
                document.fonts.add(loadedFont);
                console.log('Font registered in browser: ' + font.name);
            } catch (browserErr) {
                console.warn('Failed to register font in browser ' + font.name + ': ' + browserErr);
            }
        } catch (err) {
            console.warn('Failed to load font ' + font.name + ': ' + err);
        }
    }

    if (lastFontBytes) {
        try {
            wasmModule.register_font_bytes('sans-serif', lastFontBytes.bytes);
            console.log('Font registered as "sans-serif" (alias for Noto Sans SC)');

            const fontFaceSs = new FontFace('sans-serif', lastFontBytes.arrayBuffer);
            const loadedSsFont = await fontFaceSs.load();
            document.fonts.add(loadedSsFont);
            console.log('Font registered in browser as "sans-serif"');
        } catch (err) {
            console.warn('Failed to register fallback "sans-serif":', err);
        }
    }

    fontsReady = true;
    if (generateBtn) {
        generateBtn.disabled = false;
        generateBtn.textContent = '生成';
    }
    console.log('Font loading completed');
}

// ── Charts & Examples ─────────────────────────────────────

const chartExamples = {
    line: 'examples/line.json',
    bar: 'examples/bar.json',
    pie: 'examples/pie.json',
    area: 'examples/area.json',
    scatter: 'examples/scatter.json',
    bubble: 'examples/bubble.json',
    radar: 'examples/radar.json',
    gauge: 'examples/gauge.json',
    candlestick: 'examples/candlestick.json',
    polarBar: 'examples/polar_bar.json',
    polarScatter: 'examples/polar_scatter.json',
    table: 'examples/table.json',
    mixed: 'examples/mixed.json',
    stacked_area: 'examples/stacked_area.json',
    dual_y_axis: 'examples/dual_y_axis.json',
};

const CHART_TYPE_NAMES = {
    line: '折线图',
    bar: '柱状图',
    pie: '饼图',
    area: '面积图',
    scatter: '散点图',
    bubble: '气泡图',
    radar: '雷达图',
    gauge: '仪表盘',
    candlestick: 'K线图',
    polarBar: '极坐标柱状图',
    polarScatter: '极坐标散点图',
    table: '表格',
    mixed: '混合图',
    stacked_area: '堆叠面积图',
    dual_y_axis: '双Y轴图',
};

// ── UI Helpers ────────────────────────────────────────────

function showError(message) {
    const errorPanel = document.getElementById('errorPanel');
    errorPanel.textContent = message;
    errorPanel.classList.remove('hidden');
}

function hideError() {
    const errorPanel = document.getElementById('errorPanel');
    errorPanel.classList.add('hidden');
}

function showPlaceholder(text) {
    document.getElementById('chartContainer').innerHTML =
        '<div class="placeholder">' + text + '</div>';
    currentSvg = null;
    currentPngBytes = null;
}

// ── Core Actions ──────────────────────────────────────────

async function loadExample() {
    const chartType = document.getElementById('chartType').value;
    const examplePath = chartExamples[chartType];

    if (!examplePath) {
        setEditorValue('{\n    "error": "未知图表类型"\n}');
        return;
    }

    try {
        const response = await fetch(examplePath);
        if (!response.ok) {
            throw new Error('HTTP ' + response.status + ': ' + response.statusText);
        }
        const json = await response.text();
        setEditorValue(json);
        hideError();

        // 自动格式化
        setTimeout(formatEditor, 100);
    } catch (err) {
        showError('加载示例失败: ' + err.message);
    }
}

function injectThemeIntoJson(jsonText, themeName) {
    if (!themeName) return jsonText;

    try {
        const obj = JSON.parse(jsonText);
        obj.theme = themeName;
        return JSON.stringify(obj, null, 2);
    } catch (e) {
        return jsonText;
    }
}

async function generateChart() {
    if (!wasmModule) {
        showError('WASM 模块尚未加载完成，请稍后重试');
        return;
    }
    if (!fontsReady) {
        showError('字体正在加载中，请稍后重试');
        return;
    }

    const jsonText = getEditorValue().trim();
    if (!jsonText) {
        showError('请输入 JSON 配置');
        return;
    }

    try {
        JSON.parse(jsonText);
    } catch (e) {
        showError('JSON 格式错误: ' + e.message);
        return;
    }

    try {
        const theme = document.getElementById('themeSelect').value;
        const finalJson = injectThemeIntoJson(jsonText, theme);
        const mode = document.getElementById('renderMode').value;
        const container = document.getElementById('chartContainer');

        if (mode === 'png') {
            const pngBytes = wasmModule.render_chart_png(finalJson, 800, 600);
            currentPngBytes = pngBytes;
            currentSvg = null;

            const uint8 = new Uint8Array(pngBytes);
            const blob = new Blob([uint8], { type: 'image/png' });
            const url = URL.createObjectURL(blob);

            container.innerHTML = '<img src="' + url + '" alt="PNG Chart" style="max-width:100%;max-height:calc(100vh - 120px);border-radius:4px;box-shadow:0 2px 12px rgba(0,0,0,0.3);" />';
        } else {
            const svg = wasmModule.render_chart(finalJson, 800, 600);
            currentSvg = svg;
            currentPngBytes = null;
            container.innerHTML = svg;
        }

        hideError();
    } catch (err) {
        showError('渲染错误: ' + err);
    }
}

function downloadChart() {
    const mode = document.getElementById('renderMode').value;

    if (mode === 'svg') {
        downloadSvg();
    } else if (mode === 'png') {
        downloadPngReal();
    }
}

function downloadSvg() {
    if (!currentSvg) {
        showError('请先生成图表');
        return;
    }

    const chartType = document.getElementById('chartType').value;
    const blob = new Blob([currentSvg], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'liechart_' + chartType + '.svg';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

function downloadPngReal() {
    if (!currentPngBytes) {
        showError('请先生成图表（PNG 模式）');
        return;
    }

    const chartType = document.getElementById('chartType').value;
    const uint8 = new Uint8Array(currentPngBytes);
    const blob = new Blob([uint8], { type: 'image/png' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'liechart_' + chartType + '.png';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

function downloadPng() {
    // Legacy fallback: SVG-to-Canvas conversion for PNG mode
    if (!currentSvg) {
        showError('请先生成图表');
        return;
    }

    const chartType = document.getElementById('chartType').value;
    const svgData = currentSvg;
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');

    const tempContainer = document.createElement('div');
    tempContainer.innerHTML = svgData;
    const svgElement = tempContainer.firstElementChild;

    const svgRect = svgElement.getAttribute('viewBox') || svgElement.getAttribute('viewbox');
    let width, height;
    if (svgRect) {
        const parts = svgRect.split(/\s+/);
        width = parseInt(parts[2]);
        height = parseInt(parts[3]);
    } else {
        width = parseInt(svgElement.getAttribute('width')) || 800;
        height = parseInt (svgElement.getAttribute('height')) || 600;
    }

    canvas.width = width * 2;
    canvas.height = height * 2;
    ctx.scale(2, 2);

    const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(svgBlob);

    const img = new Image();
    img.onload = function () {
        ctx.fillStyle = '#fff';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.drawImage(img, 0, 0, width, height);
        URL.revokeObjectURL(url);

        canvas.toBlob(function (blob) {
            if (!blob) {
                showError('PNG 生成失败');
                return;
            }
            const pngUrl = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = pngUrl;
            a.download = 'liechart_' + chartType + '.png';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(pngUrl);
        }, 'image/png');
    };
    img.onerror = function () {
        showError('PNG 生成失败: 无法加载 SVG 图像');
        URL.revokeObjectURL(url);
    };
    img.src = url;
}

// ── Initialization ────────────────────────────────────────

document.addEventListener('DOMContentLoaded', function () {
    const chartTypeSelect = document.getElementById('chartType');
    const themeSelect = document.getElementById('themeSelect');
    const loadExampleBtn = document.getElementById('loadExample');
    const generateBtn = document.getElementById('generateBtn');
    const errorPanel = document.getElementById('errorPanel');
    const chartContainer = document.getElementById('chartContainer');
    const renderModeSelect = document.getElementById('renderMode');
    const downloadBtn = document.getElementById('downloadBtn');

    loadExampleBtn.addEventListener('click', loadExample);
    generateBtn.addEventListener('click', generateChart);
    downloadBtn.addEventListener('click', downloadChart);

    chartTypeSelect.addEventListener('change', function () {
        loadExample();
    });

    renderModeSelect.addEventListener('change', function () {
        const hasChart = currentSvg !== null || currentPngBytes !== null;
        if (hasChart) {
            generateChart();
        }
    });

    // 键盘快捷键: Ctrl+Enter -> 生成
    document.addEventListener('keydown', function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            generateChart();
        }
    });

    initWasm()
        .then(function () {
            loadThemes();
            return loadFonts();
        })
        .then(function () {
            return loadExample();
        })
        .then(function () {
            // 等待编辑器格式化完成后自动渲染
            setTimeout(generateChart, 300);
        })
        .catch(function (err) {
            console.error('Initialization failed:', err);
        });
});