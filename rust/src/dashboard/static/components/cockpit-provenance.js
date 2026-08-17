/* Provenance tab — durable file-touch evidence and git checkpoints. */
function ckpApi() {
  return window.LctxApi || {};
}

function ckpEscape(value) {
  return String(value == null ? '' : value).replace(/[&<>"']/g, function (ch) {
    return '&#' + ch.charCodeAt(0) + ';';
  });
}

function ckpTime(value) {
  var date = value ? new Date(value) : null;
  return date && !isNaN(date.getTime()) ? date.toLocaleString() : 'Unknown time';
}

function ckpShortSha(value) {
  var sha = String(value || 'uncommitted');
  return sha.length > 12 ? sha.slice(0, 12) : sha;
}

class CockpitProvenance extends HTMLElement {
  connectedCallback() {
    var self = this;
    this._onRefresh = function () { self.loadData(); };
    document.addEventListener('lctx:refresh', this._onRefresh);
    this.loadData();
  }

  disconnectedCallback() {
    document.removeEventListener('lctx:refresh', this._onRefresh);
  }

  async loadData() {
    var fetchJson = ckpApi().fetchJson;
    if (!fetchJson) {
      this._error = 'API client not loaded';
      this.render();
      return;
    }
    this._loading = true;
    this.render();
    try {
      var data = await fetchJson('/api/provenance', { timeoutMs: 10000 });
      if (!data || data.__error || data.error) {
        throw new Error((data && (data.__error || data.error)) || 'Failed to load provenance');
      }
      this._data = data;
      this._error = null;
    } catch (error) {
      this._error = error && error.message ? error.message : String(error || 'Failed to load provenance');
    }
    this._loading = false;
    this.render();
  }

  render() {
    var data = this._data || {};
    var checkpoints = Array.isArray(data.checkpoints) ? data.checkpoints : [];
    var records = Array.isArray(data.records) ? data.records : [];
    var esc = ckpEscape;
    if (this._loading && !this._data) {
      this.innerHTML = '<div class="ckp-state">Loading provenance…</div>';
      return;
    }
    if (this._error && !this._data) {
      this.innerHTML = '<div class="ckp-state ckp-error">' + esc(this._error) + '</div>';
      return;
    }

    var timeline = checkpoints.length ? checkpoints.map(function (checkpoint) {
      return '<article class="ckp-timeline-item">' +
        '<span class="ckp-line-dot" aria-hidden="true"></span>' +
        '<div class="ckp-timeline-body">' +
          '<div class="ckp-row"><code>' + esc(ckpShortSha(checkpoint.commit_sha)) + '</code>' +
          '<time>' + esc(ckpTime(checkpoint.observed_at)) + '</time></div>' +
          '<div class="ckp-title">Checkpoint for session <strong>' + esc(checkpoint.session_id || 'unknown') + '</strong></div>' +
          '<div class="ckp-meta"><span>' + esc(checkpoint.files_touched || 0) + ' files</span>' +
          '<span>+' + esc(checkpoint.insertions || 0) + ' / −' + esc(checkpoint.deletions || 0) + '</span>' +
          '<span>' + esc(checkpoint.link_state || 'unlinked') + '</span></div>' +
        '</div>' +
      '</article>';
    }).join('') : '<div class="ckp-empty">No recorded checkpoints yet.</div>';

    var touches = records.length ? records.map(function (record) {
      return '<tr><td><code>' + esc(record.path || 'unknown file') + '</code></td>' +
        '<td>' + esc(record.agent_id || 'unknown') + '</td>' +
        '<td>' + esc(record.session_id || 'unknown') + '</td>' +
        '<td>+' + esc(record.lines_added || 0) + ' / −' + esc(record.lines_removed || 0) + '</td>' +
        '<td><time>' + esc(ckpTime(record.observed_at)) + '</time></td></tr>';
    }).join('') : '<tr><td colspan="5" class="ckp-empty">No file touches recorded yet.</td></tr>';

    this.innerHTML = '<style>' +
      '.ckp-wrap{display:grid;gap:18px}.ckp-grid{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1.15fr);gap:18px}' +
      '.ckp-card{background:var(--surface);border:1px solid var(--border);border-radius:10px;padding:16px;min-width:0}' +
      '.ckp-card h2{font-size:14px;margin:0 0 4px;color:var(--text-bright)}.ckp-sub{color:var(--muted);font-size:12px;margin-bottom:14px}' +
      '.ckp-timeline{border-left:1px solid var(--border);margin-left:6px;padding-left:18px}.ckp-timeline-item{position:relative;padding:0 0 18px}' +
      '.ckp-line-dot{position:absolute;width:9px;height:9px;border-radius:50%;background:var(--accent);left:-23px;top:5px;box-shadow:0 0 0 3px var(--surface)}' +
      '.ckp-row,.ckp-meta{display:flex;justify-content:space-between;gap:8px;flex-wrap:wrap}.ckp-row code{color:var(--accent);font-weight:700}' +
      '.ckp-row time,.ckp-meta,time{color:var(--muted);font-size:11px}.ckp-title{margin:7px 0;color:var(--text)}.ckp-meta{font-size:12px;color:var(--muted)}' +
      '.ckp-table{width:100%;border-collapse:collapse;font-size:12px}.ckp-table th{text-align:left;color:var(--muted);font-weight:600;padding:0 8px 8px}.ckp-table td{border-top:1px solid var(--border);padding:9px 8px;vertical-align:top}.ckp-table code{color:var(--text-bright);overflow-wrap:anywhere}' +
      '.ckp-empty,.ckp-state{color:var(--muted);font-size:13px;padding:14px 0}.ckp-error{color:var(--red)}@media(max-width:900px){.ckp-grid{grid-template-columns:1fr}.ckp-table{display:block;overflow-x:auto;white-space:nowrap}}' +
      '</style><main class="ckp-wrap"><section class="ckp-card"><h2>Checkpoint timeline</h2><div class="ckp-sub">Commits linked to observed workspace changes</div><div class="ckp-timeline">' + timeline + '</div></section>' +
      '<section class="ckp-card"><h2>Recent file touches</h2><div class="ckp-sub">Observed edits with agent and session attribution</div><table class="ckp-table"><thead><tr><th>File</th><th>Agent</th><th>Session</th><th>Delta</th><th>Observed</th></tr></thead><tbody>' + touches + '</tbody></table></section></main>';
  }
}

if (!customElements.get('cockpit-provenance')) {
  customElements.define('cockpit-provenance', CockpitProvenance);
}

(function registerProvenanceLoader() {
  function register() {
    var router = window.LctxRouter;
    if (router && router.registerLoader) {
      router.registerLoader('provenance', function () {
        var element = document.getElementById('provenanceView');
        if (element && typeof element.loadData === 'function') element.loadData();
      });
    }
  }
  if (window.LctxRouter) register();
  else document.addEventListener('DOMContentLoaded', register, { once: true });
}());

export { CockpitProvenance };
