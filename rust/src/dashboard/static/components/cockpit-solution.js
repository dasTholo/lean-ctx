var SOLUTION_REFRESH_MS = 15000;

function csolEsc(value) {
  return String(value == null ? '' : value).replace(/[&<>"']/g, function (character) {
    return '&#' + character.charCodeAt(0) + ';';
  });
}

function csolNumber(value) {
  if (value && typeof value === 'object') value = value.count != null ? value.count : value.value;
  return Number(value) || 0;
}

function csolFormat(value) {
  return Math.round(csolNumber(value)).toLocaleString();
}

function csolPercent(value) {
  return Math.max(0, Math.min(100, Math.round(csolNumber(value)))) + '%';
}

function csolTrendChart(trend) {
  if (!trend.length) {
    return '<div style="color:var(--text-tertiary);font-size:12px;padding:18px 0;">No daily solution decisions recorded.</div>';
  }
  var max = Math.max.apply(null, trend.map(function (day) {
    return Math.max(csolNumber(day.decisions), Math.abs(csolNumber(day.loc_reduced)));
  }).concat([1]));
  return '<div style="display:grid;grid-template-columns:repeat(' + trend.length + ',minmax(28px,1fr));align-items:end;gap:8px;height:152px;padding-top:12px;">' + trend.map(function (day) {
    var decisions = csolNumber(day.decisions);
    var locReduced = csolNumber(day.loc_reduced);
    var decisionHeight = Math.max(4, Math.round((decisions / max) * 96));
    var locHeight = Math.max(4, Math.round((Math.abs(locReduced) / max) * 96));
    var date = String(day.date || '').slice(5) || '—';
    return '<div title="' + csolEsc(String(day.date || date) + ': ' + decisions + ' decisions, ' + locReduced + ' LOC reduced') + '" style="min-width:0;text-align:center;">' +
      '<div style="height:112px;display:flex;align-items:end;justify-content:center;gap:3px;">' +
        '<span aria-label="' + csolEsc(String(decisions) + ' decisions') + '" style="display:block;width:42%;height:' + decisionHeight + 'px;background:#4f9dff;border-radius:3px 3px 0 0;"></span>' +
        '<span aria-label="' + csolEsc(String(locReduced) + ' LOC reduced') + '" style="display:block;width:42%;height:' + locHeight + 'px;background:var(--accent-green);border-radius:3px 3px 0 0;"></span>' +
      '</div>' +
      '<div style="color:var(--text-tertiary);font-size:10px;margin-top:6px;white-space:nowrap;">' + csolEsc(date) + '</div>' +
    '</div>';
  }).join('') + '</div>' +
    '<div style="display:flex;gap:12px;color:var(--text-tertiary);font-size:10px;margin-top:8px;">' +
      '<span><i style="display:inline-block;width:7px;height:7px;background:#4f9dff;border-radius:2px;"></i> decisions</span>' +
      '<span><i style="display:inline-block;width:7px;height:7px;background:var(--accent-green);border-radius:2px;"></i> LOC reduced</span>' +
    '</div>';
}

class CockpitSolution extends HTMLElement {
  constructor() {
    super();
    this._data = null;
    this._loading = true;
    this._timer = null;
    this._onRefresh = this._onRefresh.bind(this);
  }

  connectedCallback() {
    if (this._ready) return;
    this._ready = true;
    this.addEventListener('lctx:refresh', this._onRefresh);
    this._timer = setInterval(this._onRefresh, SOLUTION_REFRESH_MS);
    this.loadData();
  }

  disconnectedCallback() {
    this.removeEventListener('lctx:refresh', this._onRefresh);
    if (this._timer) clearInterval(this._timer);
    this._timer = null;
    this._ready = false;
  }

  _onRefresh() {
    if (this.offsetParent !== null) this.loadData();
  }

  async loadData() {
    this._loading = true;
    this.render();
    try {
      var api = window.LctxApi;
      if (!api || typeof api.fetchJson !== 'function') throw new Error('Dashboard API unavailable');
      this._data = await api.fetchJson('/api/solution', { timeoutMs: 8000 });
    } catch (error) {
      this._data = { error: (error && error.message) || 'Unable to load solution intelligence' };
    }
    this._loading = false;
    this.render();
  }

  render() {
    var data = this._data || {};
    var output = data.output_savings || {};
    var loc = data.loc || {};
    var decisions = data.decisions || {};
    var trend = Array.isArray(data.trend_7d) ? data.trend_7d.slice(-7) : [];
    var topPatterns = Array.isArray(data.top_patterns) ? data.top_patterns.slice(0, 5) : [];
    var reduction = csolNumber(output.reduction_pct);
    var netReduced = csolNumber(loc.net_reduced);
    var decisionRows = [
      { label: 'STDLIB', value: csolNumber(decisions.stdlib), color: 'var(--accent-green)' },
      { label: 'REUSE', value: csolNumber(decisions.reuse), color: '#ad7cff' },
      { label: 'NATIVE', value: csolNumber(decisions.native), color: '#4f9dff' },
      { label: 'YAGNI', value: csolNumber(decisions.yagni), color: '#ff9d4f' },
      { label: 'DEBT', value: csolNumber(decisions.debt_open), color: 'var(--accent-red)' }
    ];
    var countedDecisions = decisionRows.slice(0, 4).reduce(function (total, row) { return total + row.value; }, 0);
    var totalDecisions = csolNumber(decisions.total) || countedDecisions;
    var maxDecision = Math.max.apply(null, decisionRows.map(function (row) { return row.value; }).concat([1]));
    var decisionHtml = decisionRows.map(function (row) {
      var width = Math.round((row.value / maxDecision) * 100);
      return '<div style="display:grid;grid-template-columns:70px 1fr 42px;gap:10px;align-items:center;margin:10px 0;">' +
        '<span style="color:var(--text-secondary);font-size:11px;letter-spacing:.08em;">' + row.label + '</span>' +
        '<div role="img" aria-label="' + row.label + ': ' + csolFormat(row.value) + '" style="height:8px;background:var(--border);border-radius:999px;overflow:hidden;"><div style="height:100%;width:' + width + '%;background:' + row.color + ';border-radius:inherit;"></div></div>' +
        '<span style="color:var(--text-primary);font-size:12px;text-align:right;">' + csolFormat(row.value) + '</span>' +
      '</div>';
    }).join('');
    var patternsHtml = topPatterns.length ? topPatterns.map(function (item) {
      return '<div style="display:flex;justify-content:space-between;gap:12px;margin:7px 0;font-size:12px;">' +
        '<span style="color:var(--text-secondary);overflow-wrap:anywhere;">' + csolEsc(item.pattern) + '</span>' +
        '<span style="color:var(--text-primary);">' + csolFormat(item.count) + '</span>' +
      '</div>';
    }).join('') : '<div style="color:var(--text-tertiary);font-size:12px;">No patterns recorded.</div>';
    var debtOpen = csolNumber(decisions.debt_open);
    var debtHtml = debtOpen ?
      '<div style="color:var(--accent-red);font-size:20px;font-weight:700;">' + csolFormat(debtOpen) + '</div><div style="color:var(--text-secondary);font-size:12px;margin-top:4px;">Open decision debt needs an explicit follow-up.</div>' :
      '<div style="color:var(--accent-green);font-size:14px;font-weight:600;">No active debt</div><div style="color:var(--text-secondary);font-size:12px;margin-top:4px;">All tracked decisions are resolved.</div>';
    var error = data.error ? '<div role="alert" style="margin-top:14px;color:var(--accent-red);font-size:12px;">' + csolEsc(data.error) + '</div>' : '';
    var loading = this._loading ? '<span style="color:var(--text-tertiary);font-size:11px;">REFRESHING</span>' : '';
    var intensity = String(data.intensity || 'balanced').toLowerCase();

    this.innerHTML = '<section aria-label="Solution Intelligence" style="background:var(--bg-card);border:1px solid var(--border);border-radius:12px;padding:20px;color:var(--text-primary);">' +
      '<header style="display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:18px;">' +
        '<h2 style="margin:0;font-size:13px;letter-spacing:.12em;">SOLUTION INTELLIGENCE</h2>' +
        '<label style="display:flex;align-items:center;gap:8px;color:var(--text-secondary);font-size:11px;letter-spacing:.08em;">INTENSITY ' +
          '<select aria-label="Configured solution intensity" disabled style="background:var(--bg);border:1px solid var(--accent-green);border-radius:6px;color:var(--accent-green);padding:4px 7px;text-transform:capitalize;">' +
            ['minimal', 'balanced', 'aggressive'].map(function (value) { return '<option' + (value === intensity ? ' selected' : '') + '>' + value + '</option>'; }).join('') +
          '</select></label>' + loading +
      '</header>' +
      '<div style="display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:10px;">' +
        '<div style="border:1px solid var(--border);border-radius:8px;padding:13px;"><div style="color:var(--text-tertiary);font-size:10px;letter-spacing:.08em;">OUTPUT TOKEN REDUCTION</div><div style="color:var(--accent-green);font-size:25px;font-weight:700;margin-top:6px;">-' + csolPercent(reduction) + '</div><div style="color:var(--text-tertiary);font-size:10px;margin-top:3px;">' + csolFormat(output.tokens_optimized) + ' optimized / ' + csolFormat(output.tokens_total) + '</div></div>' +
        '<div style="border:1px solid var(--border);border-radius:8px;padding:13px;"><div style="color:var(--text-tertiary);font-size:10px;letter-spacing:.08em;">NET LOC IMPACT</div><div style="font-size:25px;font-weight:700;margin-top:6px;">' + (netReduced > 0 ? '-' + csolFormat(netReduced) : '+' + csolFormat(Math.abs(netReduced))) + '</div><div style="color:var(--text-tertiary);font-size:10px;margin-top:3px;">' + csolFormat(loc.added) + ' added / ' + csolFormat(loc.removed) + ' removed</div></div>' +
        '<div style="border:1px solid var(--border);border-radius:8px;padding:13px;"><div style="color:var(--text-tertiary);font-size:10px;letter-spacing:.08em;">DECISIONS</div><div style="font-size:25px;font-weight:700;margin-top:6px;">' + csolFormat(totalDecisions) + '</div><div style="color:var(--text-tertiary);font-size:10px;margin-top:3px;">this session</div></div>' +
      '</div>' +
      '<div style="margin-top:20px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;">DECISION BREAKDOWN</div>' + decisionHtml + '</div>' +
      '<div style="display:grid;grid-template-columns:minmax(0,2fr) minmax(190px,1fr);gap:20px;margin-top:20px;">' +
        '<div style="border-top:1px solid var(--border);padding-top:14px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;">7-DAY TREND</div>' + csolTrendChart(trend) + '</div>' +
        '<div style="border-top:1px solid var(--border);padding-top:14px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;margin-bottom:8px;">ACTIVE DEBT</div>' + debtHtml + '</div>' +
      '</div>' +
      '<div style="border-top:1px solid var(--border);padding-top:14px;margin-top:20px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;margin-bottom:8px;">TOP PATTERNS</div>' + patternsHtml + '</div>' +
      error +
    '</section>';
  }
}

if (!customElements.get('cockpit-solution')) customElements.define('cockpit-solution', CockpitSolution);

(function registerSolutionLoader() {
  function doRegister() {
    var router = window.LctxRouter;
    if (!router || !router.registerLoader) return;
    router.registerLoader('solution', function () {
      var element = document.getElementById('solutionView');
      if (element && typeof element.loadData === 'function') return element.loadData();
    });
  }
  if (window.LctxRouter && window.LctxRouter.registerLoader) doRegister();
  else document.addEventListener('DOMContentLoaded', doRegister);
}());

export { CockpitSolution };
