var SOLUTION_REFRESH_MS = 15000;

function csolEsc(value) {
  return String(value == null ? '' : value).replace(/[&<>"']/g, function (character) {
    return '&#' + character.charCodeAt(0) + ';';
  });
}

function csolNumber(value) {
  if (value && typeof value === 'object') {
    value = value.count != null ? value.count : value.value;
  }
  return Number(value) || 0;
}

function csolSparkline(values) {
  var levels = '▁▂▃▄▅▆▇█';
  var max = Math.max.apply(null, values.concat([0]));
  if (!values.length) return '—';
  if (!max) return values.map(function () { return levels[0]; }).join('');
  return values.map(function (value) {
    return levels[Math.min(levels.length - 1, Math.floor((csolNumber(value) / max) * levels.length))];
  }).join('');
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
    if (this._timer) {
      clearInterval(this._timer);
      this._timer = null;
    }
    this._ready = false;
  }

  _onRefresh() {
    if (this.offsetParent !== null) this.loadData();
  }

  async loadData() {
    this._loading = true;
    this.render();
    try {
      this._data = await window.LctxApi.fetchJson('/api/solution');
    } catch (error) {
      this._data = { error: (error && error.message) || 'Unable to load solution intelligence' };
    }
    this._loading = false;
    this.render();
  }

  render() {
    var data = this._data || {};
    var metrics = data.metrics || data;
    var decisions = data.decision_breakdown || data.decisions || {};
    var output = data.output_token_savings || data.output || {};
    var loc = data.loc_metrics || data.loc || {};
    var trend = Array.isArray(data.trend_7d) ? data.trend_7d : [];
    var trendValues = trend.map(function (day) {
      return csolNumber(day.count != null ? day.count : day.decisions != null ? day.decisions : day.value);
    });
    var trendLocSaved = trend.reduce(function (total, day) {
      return total + csolNumber(day.loc_net_saved);
    }, 0);
    var topPatterns = Array.isArray(data.top_patterns) ? data.top_patterns.slice(0, 5) : [];
    var reduction = csolNumber(metrics.output_reduction_pct != null ? metrics.output_reduction_pct : output.reduction_pct);
    var netSaved = csolNumber(metrics.net_loc_saved != null ? metrics.net_loc_saved : loc.net_saved);
    var decisionRows = [
      { label: 'STDLIB', value: csolNumber(decisions.stdlib), color: 'var(--accent-green)' },
      { label: 'NATIVE', value: csolNumber(decisions.native), color: '#4f9dff' },
      { label: 'REUSE', value: csolNumber(decisions.reuse), color: '#ad7cff' },
      { label: 'YAGNI', value: csolNumber(decisions.yagni), color: '#ff9d4f' },
      { label: 'ONE-LINE', value: csolNumber(decisions.one_line != null ? decisions.one_line : decisions.oneLine), color: '#36d5d5' },
      { label: 'DEBT', value: csolNumber(decisions.debt != null ? decisions.debt : decisions.debt_open), color: 'var(--accent-red)' }
    ];
    var countedDecisions = decisionRows.reduce(function (total, row) { return total + row.value; }, 0);
    var totalDecisions = csolNumber(metrics.total_decisions != null ? metrics.total_decisions : decisions.total) || countedDecisions;
    var maxDecision = Math.max(totalDecisions, countedDecisions, 1);
    var format = function (value) { return Math.round(csolNumber(value)).toLocaleString(); };
    var pct = function (value) { return Math.round(csolNumber(value)) + '%'; };
    var decisionHtml = decisionRows.map(function (row) {
      var width = Math.min(100, (row.value / maxDecision) * 100);
      return '<div style="display:grid;grid-template-columns:78px 1fr 42px;gap:10px;align-items:center;margin:10px 0;">' +
        '<span style="color:var(--text-secondary);font-size:11px;letter-spacing:.08em;">' + row.label + '</span>' +
        '<div style="height:7px;background:var(--border);border-radius:999px;overflow:hidden;"><div style="height:100%;width:' + width + '%;background:' + row.color + ';border-radius:inherit;"></div></div>' +
        '<span style="color:var(--text-primary);font-size:12px;text-align:right;">' + format(row.value) + '</span>' +
      '</div>';
    }).join('');
    var patternsHtml = topPatterns.length ? topPatterns.map(function (pattern) {
      var label = pattern.kind != null ? pattern.kind : pattern.label;
      var count = csolNumber(pattern.count != null ? pattern.count : pattern.value);
      return '<div style="display:flex;justify-content:space-between;gap:12px;margin:7px 0;font-size:12px;">' +
        '<span style="color:var(--text-secondary);text-transform:uppercase;">' + csolEsc(label) + '</span>' +
        '<span style="color:var(--text-primary);">' + format(count) + '</span>' +
      '</div>';
    }).join('') : '<div style="color:var(--text-tertiary);font-size:12px;">No patterns recorded</div>';
    var error = data.error ? '<div style="margin:12px 0 0;color:var(--accent-red);font-size:12px;">' + csolEsc(data.error) + '</div>' : '';
    var loading = this._loading ? '<span style="color:var(--text-tertiary);font-size:11px;">REFRESHING</span>' : '';

    this.innerHTML = '<section style="background:var(--bg-card);border:1px solid var(--border);border-radius:12px;padding:20px;color:var(--text-primary);">' +
      '<header style="display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:18px;">' +
        '<div style="display:flex;align-items:center;gap:10px;">' +
          '<h2 style="margin:0;font-size:13px;letter-spacing:.12em;">SOLUTION INTELLIGENCE</h2>' +
          '<span style="border:1px solid var(--accent-green);color:var(--accent-green);border-radius:999px;padding:3px 8px;font-size:10px;letter-spacing:.08em;">' + csolEsc(data.intensity || 'HIGH INTENSITY') + '</span>' +
        '</div>' + loading +
      '</header>' +
      '<div style="display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:10px;">' +
        '<div style="border:1px solid var(--border);border-radius:8px;padding:13px;"><div style="color:var(--text-tertiary);font-size:10px;letter-spacing:.08em;">OUTPUT REDUCTION</div><div style="color:var(--accent-green);font-size:25px;font-weight:700;margin-top:6px;">' + pct(reduction) + '</div></div>' +
        '<div style="border:1px solid var(--border);border-radius:8px;padding:13px;"><div style="color:var(--text-tertiary);font-size:10px;letter-spacing:.08em;">NET LOC SAVED</div><div style="font-size:25px;font-weight:700;margin-top:6px;">' + format(netSaved) + '</div></div>' +
        '<div style="border:1px solid var(--border);border-radius:8px;padding:13px;"><div style="color:var(--text-tertiary);font-size:10px;letter-spacing:.08em;">TOTAL DECISIONS</div><div style="font-size:25px;font-weight:700;margin-top:6px;">' + format(totalDecisions) + '</div></div>' +
      '</div>' +
      '<div style="margin-top:20px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;">DECISION BREAKDOWN</div>' + decisionHtml + '</div>' +
      '<div style="display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin-top:20px;">' +
        '<div style="border-top:1px solid var(--border);padding-top:14px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;margin-bottom:8px;">7-DAY DECISION TREND</div><div style="color:var(--accent-green);font-family:monospace;font-size:24px;letter-spacing:.12em;line-height:1;">' + csolSparkline(trendValues) + '</div><div style="color:var(--text-tertiary);font-size:10px;margin-top:8px;">' + (trend.length ? format(trendValues.reduce(function (total, value) { return total + value; }, 0)) + ' DECISIONS · ' + format(trendLocSaved) + ' LOC SAVED' : 'NO DAILY DATA') + '</div></div>' +
        '<div style="border-top:1px solid var(--border);padding-top:14px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;margin-bottom:8px;">TOP PATTERNS</div>' + patternsHtml + '</div>' +
      '</div>' +
      '<div style="display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin-top:20px;">' +
        '<div style="border-top:1px solid var(--border);padding-top:14px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;margin-bottom:10px;">OUTPUT TOKEN SAVINGS</div><div style="display:grid;grid-template-columns:repeat(3,1fr);gap:8px;"><div><div style="color:var(--text-tertiary);font-size:10px;">BASELINE</div><div style="margin-top:3px;">' + format(output.baseline != null ? output.baseline : data.baseline_tokens) + '</div></div><div><div style="color:var(--text-tertiary);font-size:10px;">OPTIMIZED</div><div style="margin-top:3px;">' + format(output.optimized != null ? output.optimized : data.optimized_tokens) + '</div></div><div><div style="color:var(--text-tertiary);font-size:10px;">REDUCTION</div><div style="color:var(--accent-green);margin-top:3px;">' + pct(output.reduction != null ? output.reduction : reduction) + '</div></div></div></div>' +
        '<div style="border-top:1px solid var(--border);padding-top:14px;"><div style="color:var(--text-secondary);font-size:11px;letter-spacing:.1em;margin-bottom:10px;">LOC METRICS</div><div style="display:grid;grid-template-columns:repeat(3,1fr);gap:8px;"><div><div style="color:var(--text-tertiary);font-size:10px;">ADDED</div><div style="color:var(--accent-green);margin-top:3px;">+' + format(loc.added != null ? loc.added : data.loc_added) + '</div></div><div><div style="color:var(--text-tertiary);font-size:10px;">REMOVED</div><div style="color:var(--accent-red);margin-top:3px;">-' + format(loc.removed != null ? loc.removed : data.loc_removed) + '</div></div><div><div style="color:var(--text-tertiary);font-size:10px;">NET SAVED</div><div style="margin-top:3px;">' + format(loc.net_saved != null ? loc.net_saved : netSaved) + '</div></div></div></div>' +
      '</div>' + error +
    '</section>';
  }
}

customElements.define('cockpit-solution', CockpitSolution);

(function registerSolutionLoader() {
  function doRegister() {
    var R = window.LctxRouter;
    if (!R || !R.registerLoader) return;
    R.registerLoader('solution', function () {
      var el = document.getElementById('solutionView');
      if (el && typeof el.loadData === 'function') return el.loadData();
    });
  }
  if (window.LctxRouter && window.LctxRouter.registerLoader) doRegister();
  else document.addEventListener('DOMContentLoaded', doRegister);
})();

export { CockpitSolution };
