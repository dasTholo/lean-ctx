cockpit-overview.js 902L cognitive
// /Users/yvesgugger/Documents/Privat/Projects/lean-ctx/rust/src/dashboard/static/components/cockpit-overview.js
§ function  (L109-L151)
  async loadData() {
    var fetchJson = api();
    if (!fetchJson) {
      this._error = 'API client not loaded';
      this._loading = false;
      this.render();
      return;
    }
    this._loading = true;
    this._error = null;
    this.render();

    var paths = [
      '/api/stats',
      '/api/gain',
      '/api/buddy',
      '/api/session',
      '/api/slos',
      '/api/verification',
      '/api/graph/stats',
      '/api/roi',
      '/api/spend',
      '/api/workspaces',
      '/api/kernel',
    ];

    var cached = window.LctxApi && window.LctxApi.cachedFetch ? window.LctxApi.cachedFetch : fetchJson;
    var results = await Promise.all(
      paths.map(function (p) {
        var fn = (p === '/api/stats' || p === '/api/session') ? cached : fetchJson;
        return fn(p, { timeoutMs: 12000 }).catch(function (e) {
          return { __error: e && e.error ? e.error : String(e || 'error'), __path: p };
        });
      })
    );

    var err = [results[0], results[1]].find(function (x) {
      return x && x.__error;
    });
    if (err) {
      this._error = String(err.__path) + ': ' + String(err.__error);
    }

// ... 33 lines omitted
§ function s (L185-L223)
  render() {
    var F = fmtLib();
    var esc = F.esc || function (s) { return String(s == null ? '' : s).replace(/[&<>"']/g, function (c) { return '&#' + c.charCodeAt(0) + ';'; }); };
    var ff = F.ff || function (n) { return String(n); };
    var fmt = F.fmt || function (n) { return String(n); };
    var pc = F.pc || function (a, b) { return b > 0 ? Math.round((a / b) * 100) : 0; };
    var fu = F.fu || function (a) { return '$' + Number(a).toFixed(2); };

    if (this._loading) {
      this.innerHTML =
        '<div class="card"><div class="loading-state">Loading overview\u2026</div></div>';
      return;
    }

    if (this._error && !this._data.stats) {
      this.innerHTML =
        '<div class="card"><h3>Error</h3>' +
        '<p class="hs" style="color:var(--red)">' +
        esc(String(this._error)) +
        '</p></div>';
      return;
    }

    // Slim Home (GL #486): status, receipt, gauge+triage, one trend, top-3.
    // Deeper charts/tables live in the job areas (Proof → Trends, ROI & Plan).
    var body = '';
    body += this._renderTimeFilter(esc);
    body += this._renderHero(esc, ff, fmt, fu, pc);
    body += this._renderBuddy(esc);
    body += this._renderStatusStrip(esc);
    body += this._renderWorkspaces(esc, fmt);
    body += this._renderTrendRow();
    body += this._renderCommandTable(esc, ff, fmt, pc);

    this.innerHTML = body;
    this._bind();
    this._bindContextHealthCard();
    this._bindVerifiedBridge();
  }
// ... 32 lines omitted
§ function  (L256-L341)
  _renderHero(esc, ff, fmt, fu, pc) {
    var stats = this._data.stats;
    var gain = this._data.gain;

    var F = fmtLib();
    var fe = F.fe || function () { return '0 Wh'; };
    var ewh = F.ewh || function () { return 0; };

    var totalIn = stats ? stats.total_input_tokens || 0 : 0;
    var totalOut = stats ? stats.total_output_tokens || 0 : 0;
    var saved = totalIn - totalOut;
    var roiObj = this._data && this._data.roi && this._data.roi.roi;
    var verifiedSaved = roiObj ? roiObj.net_saved_tokens || 0 : 0;
    var compRate = verifiedSaved > 0 && totalIn > 0
      ? Math.min(100, pc(verifiedSaved, totalIn))
      : (totalIn > 0 ? pc(saved, totalIn) : 0);
    var calls = stats ? stats.total_commands || 0 : 0;
    var energyWh = ewh(saved);
    var avoidedUsd = gain && gain.summary ? gain.summary.avoided_usd || 0 : 0;
    var scoreTotal = gain && gain.summary && gain.summary.score
      ? gain.summary.score.total || 0 : 0;

    var scoreDash = Math.max(0, Math.min(100, scoreTotal));
    var scoreGap = 100 - scoreDash;
    var scoreCol = scoreDash >= 80
      ? 'var(--green)' : scoreDash >= 50
        ? 'var(--yellow)' : 'var(--red)';

    var sinceStr = stats && stats.first_use
      ? String(stats.first_use).slice(0, 10) : '';

    return (
      '<div class="hero stagger">' +

      '<div class="hero-main">' +
      '<span class="hl">Total tokens saved' + tip('total_tokens_saved') +
      '<span class="tag tb" style="margin-left:8px">estimated' +
      (sinceStr ? ' \u00b7 since ' + esc(sinceStr) : '') + '</span></span>' +
      '<div class="hv" id="cko-vSaved">' + esc(ff(saved)) + '</div>' +
      '<p class="hs">' +
      'From <b>' + esc(ff(totalIn)) + '</b> input to <b>' +
      esc(ff(totalOut)) + '</b> output across <b>' +
      esc(ff(calls)) + '</b> calls</p>' +
      this._verifiedBridge(esc, ff, fu) +
      '</div>' +

      '<div class="hc">' +
      '<span class="hl">Cost saved' + tip('cost_saved') + '</span>' +
      '<div class="hv">' + esc(fu(avoidedUsd)) + '</div>' +
      // Input-side only — the cost analysis card below adds the estimated
      // output savings on top, so the two figures intentionally differ.
      '<p class="hs">estimated input cost avoided</p>' +
      '</div>' +

      this._measuredSpendCard(esc, fu) +

      '<div class="hc">' +
      '<span class="hl">Energy saved' + tip('energy_saved') + '</span>' +
      '<div class="hv">' + esc(fe(energyWh)) + '</div>' +
      '<p class="hs">est. inference energy not burned</p>' +
      '</div>' +

      '<div class="hc">' +
      '<span class="hl">Compression rate' + tip('compression_rate') + '</span>' +
      '<div class="hv">' + esc(String(compRate)) + '%</div>' +
      '<p class="hs">tokens removed before sending</p>' +
      '</div>' +

      '<div class="hc">' +
      '<span class="hl">Gain score' + tip('gain_score') + '</span>' +
      (window.LctxShared && window.LctxShared.gaugeRing
        ? window.LctxShared.gaugeRing(scoreDash, scoreCol, 72, Math.round(scoreTotal))
        : '<div class="gauge-ring" style="width:72px;height:72px"><span class="gauge-value">' + Math.round(scoreTotal) + '</span></div>') +
      '</div>' +

      '<div class="hc">' +
      '<span class="hl">Total calls' + tip('total_calls') + '</span>' +
      '<div class="hv">' + esc(ff(calls)) + '</div>' +
      '<p class="hs">' +
      (stats && stats.first_use
        ? 'since ' + esc(String(stats.first_use).slice(0, 10))
        : '') +
      '</p>' +
      '</div>' +

      this._healthHeroCard(esc, ff) +

      '</div>'
    );
  }
// ... 22 lines omitted
§ function function (L364-L387)
  _verifiedBridge(esc, ff, fu) {
    var roiPayload = this._data && this._data.roi;
    var roi = roiPayload && roiPayload.roi ? roiPayload.roi : null;
    if (!roi || !roi.total_events) return '';

    var trend = roiPayload.trend || [];
    var since = trend.length && trend[0] && trend[0][0] ? String(trend[0][0]) : '';
    var verificationTag = '<span class="tag ty">unsigned</span>';
    if (roi.chain_valid === false) {
      verificationTag = '<span class="tag td">chain BROKEN</span>';
    } else if (roi.chain_valid && roi.signed) {
      verificationTag = '<span class="tag tg">verified</span>';
    }

    return (
      '<p class="hs cko-bridge" id="cko-verifiedBridge" role="link" tabindex="0" ' +
      'title="Open ROI & Plan" style="cursor:pointer;margin-top:6px">' +
      verificationTag + ' ' +
      '<b>' + esc(ff(roi.net_saved_tokens)) + '</b> net tokens saved \u00b7 <b>' +
      esc(fu(roi.saved_usd)) + '</b> \u00b7 separate measured ledger' +
      (since ? ' (recording since ' + esc(since) + ')' : '') +
      ' <span class="hc-health-go">ROI &amp; Plan \u2192</span></p>'
    );
  }
// ... 87 lines omitted
§ function function (L475-L537)
  _renderBuddy(esc) {
    var b = this._data.buddy;
    if (!b || !b.name) return '';

    var rarity = b.rarity || 'Common';
    var rarityLabel = rarity === 'Egg' ? 'Starter' : rarity;
    var tier = lvlTier(b.level || 1);
    var art = Array.isArray(b.ascii_art) ? b.ascii_art.join('\n') : (b.ascii_art || '');
    var mood = b.mood || 'Content';
    // Coherent, endless progression: the form follows the evolution\u2192ascension
    // ladder (never a dead-end word), and the themed aura intensifies with each
    // ascension tier so the buddy keeps visibly changing forever.
    var form = b.form || 'Egg';
    var prestige = b.prestige || 0;
    var glow = 12 + Math.min(prestige, 18) * 2;
    var spriteCls = 'buddy-sprite buddy-sprite--theme ' + tier +
      (prestige > 0 ? ' buddy-sprite--ascend' : '');

    // Real lean-ctx efficiency metrics — no abstract RPG stats.
    var effMetrics = [
      { label: 'Compression', val: b.compression_pct || 0, color: 'var(--accent)', tipKey: 'compression' },
      { label: 'Cache', val: b.cache_hit_rate || 0, color: 'var(--text-bright)', tipKey: 'buddy_cache' },
    ];

    var statsHtml = '<div class="buddy-stats-grid">';
    for (var i = 0; i < effMetrics.length; i++) {
      var em = effMetrics[i];
      statsHtml +=
        '<div class="stat-cell">' +
        '<div class="stat-label">' + em.label + tip(em.tipKey) + '</div>' +
        miniGauge(em.val, em.color) +
        '<div class="stat-val">' + em.val + '%</div>' +
        '</div>';
    }
    statsHtml += '</div>';

    return (
      '<div class="buddy-card buddy-card--theme ' + tier +
      '" style="margin-bottom:20px">' +
      '<div class="' + spriteCls + '" style="--buddyGlow:' + glow + 'px">' +
      '<pre id="cko-buddyArt">' + esc(art) + '</pre>' +
      '</div>' +
      '<div class="buddy-info">' +
      '<div class="buddy-name">' + esc(b.name) +
      ' <span class="rarity-badge r-' + esc(rarity) + '">' +
      esc(rarityLabel) + '</span></div>' +
      '<div class="buddy-meta">' +
      '<span class="buddy-form">' + esc(form) + tip('buddy_form') + '</span>' +
      '<span>Lv.' + (b.level || 1) + tip('buddy_level') + '</span>' +
      '<span class="mood-dot mood-' + esc(mood) + '"></span>' +
      '<span>' + esc(mood) + tip('buddy_mood') + '</span>' +
      (b.streak_days != null
        ? '<span>' + b.streak_days + 'd streak' + tip('buddy_streak') + '</span>'
        : '') +
      '</div>' +
      statsHtml +
      (b.speech
        ? '<div class="buddy-speech">' + esc(b.speech) + '</div>'
        : '') +
      '</div>' +
      '</div>'
    );
  }
// ... 140 lines omitted
§ function  (L678-L717)
  _renderCommandTable(esc, ff, fmt, pc) {
    var stats = this._data.stats;
    var cmds = stats && stats.commands ? stats.commands : {};
    var keys = Object.keys(cmds);
    if (!keys.length) return '';

    var F = fmtLib();
    var isM = F.isM || function () { return false; };
    var sb = F.sb || function () { return ''; };

    var rows = [];
    var maxSaved = 0;
    for (var i = 0; i < keys.length; i++) {
      var name = keys[i];
      var s = cmds[name];
      var saved = (s.input_tokens || 0) - (s.output_tokens || 0);
      if (saved > maxSaved) maxSaved = saved;
      rows.push({
        name: name,
        count: s.count || 0,
        input: s.input_tokens || 0,
        output: s.output_tokens || 0,
        saved: saved,
        pct: s.input_tokens > 0 ? pc(saved, s.input_tokens) : 0,
      });
    }

    var sk = this._sortKey;
    var dir = this._sortDir === 'desc' ? -1 : 1;
    rows.sort(function (a, b) {
      var av = a[sk];
      var bv = b[sk];
      if (typeof av === 'string') av = av.toLowerCase();
      if (typeof bv === 'string') bv = bv.toLowerCase();
      if (av < bv) return -1 * dir;
      if (av > bv) return 1 * dir;
      return 0;
    });

    var sortDir = this._sortDir;
// ... 80 lines omitted
§ function function (L798-L833)
  _chartCumSavings() {
    var Ch = chartsLib();
    if (!Ch.lineChart || typeof Chart === 'undefined') return;
    var daily = this._filteredDaily();
    if (!daily.length) return;

    // Baseline so the "All" view's right edge always equals the all-time total
    // shown in the hero — even when older daily rows have aged out of retention.
    // Shorter ranges stay zero-based to show in-window growth.
    var stats = this._data && this._data.stats;
    var baseline = 0;
    if (this._range === 0 && stats) {
      var allTime = Math.max(0, (stats.total_input_tokens || 0) - (stats.total_output_tokens || 0));
      var stored = Array.isArray(stats.daily) ? stats.daily : [];
      var storedSum = 0;
      for (var j = 0; j < stored.length; j++) {
        storedSum += (stored[j].input_tokens || 0) - (stored[j].output_tokens || 0);
      }
      baseline = Math.max(0, allTime - storedSum);
    }

    var labels = [];
    var values = [];
    var cum = baseline;
    for (var i = 0; i < daily.length; i++) {
      var d = daily[i];
      labels.push(String(d.date || '').slice(5));
      cum += (d.input_tokens || 0) - (d.output_tokens || 0);
      values.push(cum);
    }

    Ch.lineChart(
      'cko-chartCumSavings', labels, values,
      '#34d399', 'rgba(52,211,153,.06)'
    );
  }
7/54 chunks shown (3552 tokens)
[lean-ctx] full source: read "/Users/yvesgugger/Documents/Privat/Projects/lean-ctx/rust/src/dashboard/static/components/cockpit-overview.js" directly (no MCP)  ·  or ctx_read("/Users/yvesgugger/Documents/Privat/Projects/lean-ctx/rust/src/dashboard/static/components/cockpit-overview.js", mode="full")
