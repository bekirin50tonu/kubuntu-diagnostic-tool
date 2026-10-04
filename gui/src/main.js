import { invoke } from '@tauri-apps/api/core';

const state = {
    report: null,
    systemInfo: null,
    currentView: 'dashboard',
};

function $(sel) {
    const root = document.getElementById('app');
    const el = root.querySelector(sel);
    if (el) return el;
    const div = document.createElement('template');
    div.innerHTML = sel.startsWith('<') ? sel : `<div>${sel}</div>`;
    return div.content.firstElementChild;
}

function createElement(html) {
    const t = document.createElement('template');
    t.innerHTML = html.trim();
    return t.content.firstElementChild;
}

function severityClass(s) {
    const lower = (s || '').toLowerCase();
    if (lower.includes('critical')) return 'severity-critical';
    if (lower.includes('error')) return 'severity-error';
    if (lower.includes('warning')) return 'severity-warning';
    return 'severity-ok';
}

function severityBadge(s, label) {
    const lower = (s || '').toLowerCase();
    let cls = 'badge-';
    if (lower.includes('critical')) cls += 'critical';
    else if (lower.includes('error')) cls += 'error';
    else if (lower.includes('warning')) cls += 'warning';
    else return `<span class="badge" style="background:#333;color:#aaa">OK</span>`;
    return `<span class="badge ${cls}">${label || lower}</span>`;
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function formatDuration(seconds) {
    const d = Math.floor(seconds / 86400);
    const h = Math.floor((seconds % 86400) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    if (d > 0) return `${d}d ${h}h ${m}m`;
    if (h > 0) return `${h}h ${m}m ${s}s`;
    return `${m}m ${s}s`;
}

// ---------- Layout ----------
function renderLayout() {
    const app = document.getElementById('app');
    app.innerHTML = `
        <div class="app">
            <div class="sidebar">
                <h1>KDiag</h1>
                <div class="nav-item active" data-view="dashboard">📊 Dashboard</div>
                <div class="nav-item" data-view="warnings">⚠️  Warnings & Errors <span id="warn-count" class="sev"></span></div>
                <div class="nav-item" data-view="services">🔧 Services</div>
                <div class="nav-item" data-view="system">💻 System Info</div>
                <div class="nav-item" data-view="report">📋 Full Report</div>
                <div style="flex:1"></div>
                <div style="padding:10px 20px;color:var(--text-muted);font-size:12px">
                    <button class="btn" onclick="location.reload()">Refresh</button>
                </div>
            </div>
            <div class="content" id="view-content">
                <div class="header">
                    <h2 id="view-title">Dashboard</h2>
                    <button id="scan-btn" class="btn primary">🔄 Scan Now</button>
                </div>
                <div id="view-body"></div>
            </div>
        </div>
    `;

    // Navigation
    app.querySelectorAll('.nav-item').forEach(item => {
        item.addEventListener('click', () => {
            app.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
            item.classList.add('active');
            switchView(item.dataset.view);
        });
    });

    document.getElementById('scan-btn').addEventListener('click', () => scan());
}

function switchView(view) {
    state.currentView = view;
    const titles = { dashboard: 'Dashboard', warnings: 'Warnings & Errors', services: 'Services', system: 'System Info', report: 'Full Report' };
    document.getElementById('view-title').textContent = titles[view] || view;
    renderView(view);
}

function renderView(view) {
    const body = document.getElementById('view-body');
    switch (view) {
        case 'dashboard': renderDashboard(body); break;
        case 'warnings': renderWarnings(body); break;
        case 'services': renderServices(body); break;
        case 'system': renderSystem(body); break;
        case 'report': renderReport(body); break;
    }
}

// ---------- Scanning ----------
async function scan() {
    const btn = document.getElementById('scan-btn');
    btn.textContent = '⏳ Scanning...';
    btn.disabled = true;
    try {
        const [report, sysinfo] = await Promise.all([
            invoke('run_diagnostics'),
            invoke('get_system_info'),
        ]);
        state.report = report;
        state.systemInfo = sysinfo;
        renderView(state.currentView);
        updateWarnCount();
    } catch (e) {
        console.error(e);
        document.getElementById('view-body').innerHTML = `<div style="color:var(--error)">Scan failed: ${e}</div>`;
    } finally {
        btn.textContent = '🔄 Scan Now';
        btn.disabled = false;
    }
}

function updateWarnCount() {
    const el = document.getElementById('warn-count');
    if (!state.report) return;
    const warnings = state.report.results.filter(r => r.severity.toLowerCase() !== 'ok');
    el.textContent = warnings.length ? `(${warnings.length})` : '';
}

// ---------- Dashboard ----------
function renderDashboard(body) {
    if (!state.systemInfo) {
        body.innerHTML = '<p style="color:var(--text-muted);padding:40px">Click <strong>Scan Now</strong> to load system data.</p>';
        return;
    }
    const info = state.systemInfo;
    body.innerHTML = `
        <div class="cards">
            <div class="card"><div class="card-title">CPU</div>
                <div class="card-value ${severityClass(info.cpu_usage > 85 ? 'Warning' : '')}">${info.cpu_usage.toFixed(1)}%</div>
                <div class="card-sub">${info.cpu_brand} · ${info.cpu_count} cores</div>
            </div>
            <div class="card"><div class="card-title">RAM</div>
                <div class="card-value">${formatBytes(info.used_memory)}</div>
                <div class="card-sub">of ${formatBytes(info.total_memory)} (${((info.used_memory / info.total_memory) * 100).toFixed(1)}%)</div>
            </div>
            <div class="card"><div class="card-title">Swap</div>
                <div class="card-value">${formatBytes(info.used_swap)}</div>
                <div class="card-sub">of ${formatBytes(info.total_swap)} (${((info.used_swap / info.total_swap) * 100).toFixed(1)}%)</div>
            </div>
            <div class="card"><div class="card-title">Disk</div>
                <div class="card-value">${formatBytes(info.used_disk)}</div>
                <div class="card-sub">of ${formatBytes(info.total_disk)} (${((info.used_disk / info.total_disk) * 100).toFixed(1)}%)</div>
            </div>
            <div class="card"><div class="card-title">Uptime</div>
                <div class="card-value" style="font-size:22px">${info.uptime}</div>
                <div class="card-sub">${info.os_version || 'Linux'}</div>
            </div>
            <div class="card"><div class="card-title">Host</div>
                <div class="card-value" style="font-size:20px">${info.hostname}</div>
                <div class="card-sub">${info.kernel_version}</div>
            </div>
        </div>
        ${state.report ? renderReportSummary(state.report) : ''}
    `;
}

function renderReportSummary(report) {
    const nonOk = report.results.filter(r => r.severity.toLowerCase() !== 'ok');
    return `
        <h3 style="margin-top:24px;margin-bottom:12px">Check Results</h3>
        <div style="margin-bottom:8px">Overall: ${severityBadge(report.overall_severity, report.overall_severity)}</div>
        ${report.results.map(r => `
            <div class="report-line">
                ${severityBadge(r.severity, '')}
                <strong>${r.check_name}</strong> — ${r.message}
                ${r.details ? `<span style="color:var(--text-muted)">· ${r.details}</span>` : ''}
            </div>
        `).join('')}
    `;
}

// ---------- Warnings & Errors ----------
function renderWarnings(body) {
    if (!state.report) {
        body.innerHTML = '<p style="color:var(--text-muted);padding:40px">Run a scan first.</p>';
        return;
    }
    const errors = state.report.results.filter(r => r.severity.toLowerCase() === 'error' || r.severity.toLowerCase() === 'critical');
    const warnings = state.report.results.filter(r => r.severity.toLowerCase() === 'warning');

    // Build copyable text
    const problemLines = [...errors, ...warnings].map(r =>
        `[${r.severity.toUpperCase()}] ${r.check_name}: ${r.message}${r.details ? ` — ${r.details}` : ''}`
    ).join('\n');
    const fullReportText = state.report.results.map(r =>
        `[${r.severity.toUpperCase()}] ${r.check_name}: ${r.message}${r.details ? ` — ${r.details}` : ''}`
    ).join('\n');

    body.innerHTML = `
        <div class="warnings-section">
            <div class="section-title">🔴 Errors ${errors.length ? `<span class="badge badge-error">${errors.length}</span>` : ''}</div>
            ${errors.length ? errors.map(r => `
                <div class="report-line">
                    ${severityBadge(r.severity, r.severity)} <strong>${r.check_name}</strong> — ${r.message}
                    ${r.details ? `<span style="color:var(--text-muted)">· ${r.details}</span>` : ''}
                </div>
            `).join('') : '<p style="color:var(--text-muted)">No errors found.</p>'}
        </div>
        <div class="errors-section">
            <div class="section-title">⚠️  Warnings ${warnings.length ? `<span class="badge badge-warning">${warnings.length}</span>` : ''}</div>
            ${warnings.length ? warnings.map(r => `
                <div class="report-line">
                    ${severityBadge(r.severity, r.severity)} <strong>${r.check_name}</strong> — ${r.message}
                    ${r.details ? `<span style="color:var(--text-muted)">· ${r.details}</span>` : ''}
                </div>
            `).join('') : '<p style="color:var(--text-muted)">No warnings.</p>'}
        </div>
        <div style="margin-top:24px;border-top:1px solid var(--border);padding-top:16px">
            <div class="section-title">📋 Copy Problem Summary</div>
            <textarea id="problem-text" rows="8" readonly>${problemLines}</textarea>
            <button class="btn copy-btn" onclick="copyText('problem-text')">📋 Copy to Clipboard</button>
            <div style="margin-top:12px">
                <div class="section-title">🤖 Get AI Help</div>
                <div style="color:var(--text-muted);margin-bottom:8px;font-size:13px">
                    Paste the problem summary into your AI assistant to get troubleshooting help.
                </div>
                <div style="display:flex;gap:8px">
                    <textarea id="ai-query" rows="3" placeholder="Optional: describe your issue in your own words..." style="flex:1">Fix these Kubuntu system issues:\n\n${problemLines}</textarea>
                    <button class="btn" onclick="copyAIPrompt()" style="white-space:nowrap">📋 Copy Prompt</button>
                </div>
            </div>
            <div style="margin-top:12px">
                <div class="section-title">📄 Full Report</div>
                <textarea id="full-report-text" rows="6" readonly>${fullReportText}</textarea>
                <button class="btn copy-btn" onclick="copyText('full-report-text')">📋 Copy Full Report</button>
            </div>
        </div>
    `;
}

window.copyText = function(id) {
    const el = document.getElementById(id);
    el.select();
    navigator.clipboard.writeText(el.value).then(() => {
        el.style.outline = '2px solid var(--ok)';
        setTimeout(() => el.style.outline = '', 1500);
    });
};

window.copyAIPrompt = function() {
    const el = document.getElementById('ai-query');
    el.select();
    navigator.clipboard.writeText(el.value).then(() => {
        el.style.outline = '2px solid var(--ok)';
        setTimeout(() => el.style.outline = '', 1500);
    });
};

// ---------- Services ----------
function renderServices(body) {
    if (!state.report) {
        body.innerHTML = '<p style="color:var(--text-muted);padding:40px">Run a scan first.</p>';
        return;
    }
    body.innerHTML = `
        <div style="margin-bottom:16px">
            <button class="btn primary" onclick="loadServices()">🔄 List Failed Services</button>
        </div>
        <div id="services-table"></div>
    `;
}

window.loadServices = async function() {
    const container = document.getElementById('services-table');
    container.innerHTML = '<p style="color:var(--text-muted)">Loading...</p>';
    try {
        const services = await invoke('get_failed_services');
        if (services.length === 0) {
            container.innerHTML = '<p style="color:var(--ok)">✅ No failed services.</p>';
            return;
        }
        container.innerHTML = `
            <div class="table-container">
                <table>
                    <thead><tr><th>Unit</th><th>Action</th></tr></thead>
                    <tbody>
                        ${services.map(s => `
                            <tr>
                                <td><span style="color:var(--error)">●</span> ${s}</td>
                                <td>
                                    <button class="btn" onclick="serviceAction('status','${s}')">Status</button>
                                    <button class="btn" onclick="serviceAction('start','${s}')">Start</button>
                                    <button class="btn" onclick="serviceAction('restart','${s}')">Restart</button>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            </div>
            <div id="service-output" style="margin-top:12px"></div>
        `;
    } catch (e) {
        container.innerHTML = `<p style="color:var(--error)">${e}</p>`;
    }
};

window.serviceAction = async function(action, unit) {
    const output = document.getElementById('service-output');
    output.innerHTML = '<p style="color:var(--text-muted)">Running...</p>';
    try {
        const result = await invoke('run_systemctl_command', { action, unit });
        output.innerHTML = `<pre style="background:#111;padding:12px;border-radius:6px;font-family:monospace;font-size:12px;overflow-x:auto">${result}</pre>`;
    } catch (e) {
        output.innerHTML = `<p style="color:var(--error)">${e}</p>`;
    }
};

// ---------- System Info ----------
function renderSystem(body) {
    if (!state.systemInfo) {
        body.innerHTML = '<p style="color:var(--text-muted);padding:40px">Run a scan first.</p>';
        return;
    }
    const info = state.systemInfo;
    body.innerHTML = `
        <div class="cards">
            <div class="card"><div class="card-title">Hostname</div><div class="card-value" style="font-size:22px">${info.hostname}</div></div>
            <div class="card"><div class="card-title">OS</div><div class="card-value" style="font-size:18px">${info.os_version || 'Unknown'}</div></div>
            <div class="card"><div class="card-title">Kernel</div><div class="card-value" style="font-size:18px">${info.kernel_version}</div></div>
            <div class="card"><div class="card-title">Uptime</div><div class="card-value">${info.uptime}</div></div>
            <div class="card"><div class="card-title">CPU</div>
                <div class="card-value" style="font-size:20px">${info.cpu_brand}</div>
                <div class="card-sub">${info.cpu_count} cores · ${info.cpu_usage.toFixed(1)}% used</div>
            </div>
            <div class="card"><div class="card-title">Memory</div>
                <div class="card-value">${formatBytes(info.total_memory)}</div>
                <div class="card-sub">${formatBytes(info.used_memory)} used · swap: ${formatBytes(info.total_swap)}</div>
            </div>
            <div class="card"><div class="card-title">Disk</div>
                <div class="card-value">${formatBytes(info.total_disk)}</div>
                <div class="card-sub">${formatBytes(info.used_disk)} used · ${formatBytes(info.total_disk - info.used_disk)} free</div>
            </div>
        </div>
    `;
}

// ---------- Full Report ----------
function renderReport(body) {
    if (!state.report) {
        body.innerHTML = '<p style="color:var(--text-muted);padding:40px">Run a scan first.</p>';
        return;
    }
    const jsonStr = JSON.stringify(state.report, null, 2);
    body.innerHTML = `
        <div style="margin-bottom:12px">
            <button class="btn" onclick="window.copyReport()">📋 Copy JSON</button>
        </div>
        <pre style="background:#111;padding:16px;border-radius:8px;overflow:auto;max-height:70vh;font-family:monospace;font-size:12px;line-height:1.6">${jsonStr}</pre>
    `;
}

window.copyReport = function() {
    const text = JSON.stringify(state.report, null, 2);
    navigator.clipboard.writeText(text);
};

// ---------- Init ----------
renderLayout();
scan(); // Auto-scan on load