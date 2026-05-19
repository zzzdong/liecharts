let wasmModule = null;
let fontsReady = false;
let monacoReady = false;
let editorInstance = null;

const FONTS_TO_LOAD = [
    {
        name: 'JetBrains Mono',
        url: 'https://cdn.jsdelivr.net/gh/JetBrains/JetBrainsMono@v2.304/fonts/ttf/JetBrainsMono-Regular.ttf',
    },
    {
        name: 'Noto Sans SC',
        url: 'https://fonts.gstatic.com/s/notosanssc/v26/k3kXo84MPvpLmixcA63oeALhL4iP-Q8.otf',
    },
];

const THEME_NAMES = {
    echarts: 'ECharts 6',
    light: 'Light',
    dark: 'Dark',
    vintage: 'Vintage',
    macarons: 'Macarons',
    infographic: 'Infographic',
    shine: 'Shine',
    roma: 'Roma',
};

let currentSvg = null;
let currentPngBytes = null;

// ── Monaco Editor ──────────────────────────────────────────

require.config({
    paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.0/min/vs' },
});

require(['vs/editor/editor.main'], function () {
    monacoReady = true;

    monaco.editor.defineTheme('liecharts-light', {
        base: 'vs',
        inherit: true,
        rules: [
            { token: 'string.key.json', foreground: '#5641b3' },
            { token: 'string.value.json', foreground: '#15803d' },
            { token: 'number.json', foreground: '#b91c1c' },
            { token: 'keyword.json', foreground: '#6e5cbe' },
            { token: 'delimiter.json', foreground: '#5641b3' },
        ],
        colors: {
            'editor.background': '#f9f9f9',
            'editor.foreground': '#1f2329',
            'editorLineNumber.foreground': '#9fa2a7',
            'editorCursor.foreground': '#6e5cbe',
            'editor.selectionBackground': '#6e5cbe20',
            'editor.lineHighlightBackground': '#f4f4f5',
            'editorIndentGuide.background': '#e6e8ea',
            'editorIndentGuide.activeBackground': '#d1d3d6',
            'editorBracketMatch.background': '#6e5cbe15',
            'editorBracketMatch.border': '#6e5cbe',
            'editorWidget.background': '#ffffff',
            'editorWidget.border': '#e6e8ea',
            'minimap.background': '#f9f9f9',
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
        theme: 'liecharts-light',
        fontSize: 13,
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
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
        padding: { top: 12, bottom: 12 },
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
    monaco.editor.setModelMarkers(editorInstance.getModel(), 'json', markers);
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
        console.log('WASM module loaded');
    } catch (err) {
        console.error('WASM load failed:', err);
        showError('WASM module failed to load: ' + err.message);
        throw err;
    }
}

async function loadThemes() {
    if (!wasmModule) return;

    try {
        const themesJson = wasmModule.get_available_themes();
        const themes = JSON.parse(themesJson);
        const select = document.getElementById('themeSelect');
        select.innerHTML = '<option value="">Default</option>';

        themes.forEach(function (name) {
            const displayName = THEME_NAMES[name] || name;
            const opt = document.createElement('option');
            opt.value = name;
            opt.textContent = displayName;
            select.appendChild(opt);
        });

        console.log('Themes:', themes.join(', '));
    } catch (err) {
        console.warn('Failed to load themes:', err);
    }
}

async function loadFonts() {
    const generateBtn = document.getElementById('generateBtn');
    let lastFontBytes = null;

    for (const font of FONTS_TO_LOAD) {
        try {
            console.log('Loading font:', font.name);
            const response = await fetch(font.url);
            if (!response.ok) {
                throw new Error('HTTP ' + response.status);
            }
            const arrayBuffer = await response.arrayBuffer();
            const bytes = new Uint8Array(arrayBuffer);

            lastFontBytes = { arrayBuffer: arrayBuffer, bytes: bytes };

            wasmModule.register_font_bytes(font.name, bytes);

            try {
                const fontFace = new FontFace(font.name, arrayBuffer);
                const loadedFont = await fontFace.load();
                document.fonts.add(loadedFont);
                console.log('Font loaded:', font.name);
            } catch (browserErr) {
                console.warn('Browser font load failed:', font.name, browserErr);
            }
        } catch (err) {
            console.warn('Font load failed:', font.name, err);
        }
    }

    if (lastFontBytes) {
        try {
            wasmModule.register_font_bytes('sans-serif', lastFontBytes.bytes);
            const fontFaceSs = new FontFace('sans-serif', lastFontBytes.arrayBuffer);
            const loadedSsFont = await fontFaceSs.load();
            document.fonts.add(loadedSsFont);
        } catch (err) {
            console.warn('Fallback font failed:', err);
        }
    }

    fontsReady = true;
    if (generateBtn) {
        generateBtn.disabled = false;
    }
    console.log('Font loading done');
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

// ── UI Helpers ────────────────────────────────────────────

function showError(message) {
    const errorPanel = document.getElementById('errorPanel');
    const errorMsg = errorPanel.querySelector('.error-message');
    errorMsg.textContent = message;
    errorPanel.classList.remove('hidden');
}

function hideError() {
    const errorPanel = document.getElementById('errorPanel');
    errorPanel.classList.add('hidden');
}

function showPlaceholder(text) {
    document.getElementById('chartContainer').innerHTML =
        '<div class="empty-state"><svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/></svg><p>' + text + '</p></div>';
    currentSvg = null;
    currentPngBytes = null;
}

function setLoading(loading) {
    const btn = document.getElementById('generateBtn');
    const spinner = btn.querySelector('.btn-spinner');
    const text = btn.querySelector('.btn-text');
    if (loading) {
        spinner.classList.remove('hidden');
        text.textContent = 'Rendering...';
        btn.disabled = true;
    } else {
        spinner.classList.add('hidden');
        text.textContent = 'Render';
        btn.disabled = false;
    }
}

// ── Core Actions ──────────────────────────────────────────

async function loadExample() {
    const chartType = document.getElementById('chartType').value;
    const examplePath = chartExamples[chartType];

    if (!examplePath) {
        setEditorValue('{\n    "error": "Unknown chart type"\n}');
        return;
    }

    try {
        const response = await fetch(examplePath);
        if (!response.ok) {
            throw new Error('HTTP ' + response.status);
        }
        const json = await response.text();
        setEditorValue(json);
        hideError();
        setTimeout(formatEditor, 100);
    } catch (err) {
        showError('Failed to load example: ' + err.message);
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
        showError('WASM module not ready');
        return;
    }
    if (!fontsReady) {
        showError('Fonts still loading');
        return;
    }

    const jsonText = getEditorValue().trim();
    if (!jsonText) {
        showError('Please enter JSON configuration');
        return;
    }

    try {
        JSON.parse(jsonText);
    } catch (e) {
        showError('Invalid JSON: ' + e.message);
        return;
    }

    setLoading(true);

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

            container.innerHTML = '<img src="' + url + '" alt="Chart" />';
        } else {
            const svg = wasmModule.render_chart(finalJson, 800, 600);
            currentSvg = svg;
            currentPngBytes = null;
            container.innerHTML = svg;
        }

        hideError();
    } catch (err) {
        showError('Render error: ' + err);
    } finally {
        setLoading(false);
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
        showError('Please render a chart first');
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
        showError('Please render a chart in PNG mode first');
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
        showError('Please render a chart first');
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
        height = parseInt(svgElement.getAttribute('height')) || 600;
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
                showError('PNG generation failed');
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
        showError('PNG generation failed: cannot load SVG');
        URL.revokeObjectURL(url);
    };
    img.src = url;
}

// ── Resize Handle ─────────────────────────────────────────

function initResizeHandle() {
    const handle = document.getElementById('resizeHandle');
    const editorPanel = document.querySelector('.panel-editor');
    let isDragging = false;

    handle.addEventListener('mousedown', (e) => {
        isDragging = true;
        handle.classList.add('dragging');
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
        e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        const workspace = document.querySelector('.workspace');
        const rect = workspace.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const pct = (x / rect.width) * 100;
        editorPanel.style.flex = 'none';
        editorPanel.style.width = Math.max(20, Math.min(80, pct)) + '%';
    });

    document.addEventListener('mouseup', () => {
        if (isDragging) {
            isDragging = false;
            handle.classList.remove('dragging');
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
    });
}

// ── Initialization ────────────────────────────────────────

document.addEventListener('DOMContentLoaded', function () {
    const chartTypeSelect = document.getElementById('chartType');
    const themeSelect = document.getElementById('themeSelect');
    const loadExampleBtn = document.getElementById('loadExample');
    const generateBtn = document.getElementById('generateBtn');
    const renderModeSelect = document.getElementById('renderMode');
    const downloadBtn = document.getElementById('downloadBtn');

    loadExampleBtn.addEventListener('click', loadExample);
    generateBtn.addEventListener('click', generateChart);
    downloadBtn.addEventListener('click', downloadChart);

    chartTypeSelect.addEventListener('change', function () {
        loadExample();
    });

    themeSelect.addEventListener('change', function () {
        const hasChart = currentSvg !== null || currentPngBytes !== null;
        if (hasChart) {
            generateChart();
        }
    });

    renderModeSelect.addEventListener('change', function () {
        const hasChart = currentSvg !== null || currentPngBytes !== null;
        if (hasChart) {
            generateChart();
        }
    });

    // Keyboard shortcut: Ctrl+Enter -> Render
    document.addEventListener('keydown', function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            generateChart();
        }
    });

    initResizeHandle();

    initWasm()
        .then(function () {
            loadThemes();
            return loadFonts();
        })
        .then(function () {
            return loadExample();
        })
        .then(function () {
            setTimeout(generateChart, 300);
        })
        .catch(function (err) {
            console.error('Initialization failed:', err);
        });
});