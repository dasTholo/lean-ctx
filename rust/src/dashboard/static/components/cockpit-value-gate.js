/** ValueGate outcome quality and CPAO dashboard panel. */
class CockpitValueGate extends HTMLElement {
  constructor() { super(); this.data = null; this.loading = true; }

  connectedCallback() {
    if (this.ready) return;
    this.ready = true; this.style.display = 'block'; this.render();
    document.addEventListener('lctx:refresh', () => this.loadData());
  }

  async loadData() {
    this.loading = true; this.render();
    try {
      const response = await fetch('/api/value-gate/summary');
      this.data = response.ok ? await response.json() : { recent_assessments: [], aggregate: {} };
    } catch (_) { this.data = { recent_assessments: [], aggregate: {} }; }
    this.loading = false; this.render();
  }

  money(micros) { return `$${(Number(micros || 0) / 1000000).toFixed(4)}`; }
  render() {
    if (this.loading) { this.innerHTML = '<div class="card"><div class="loading-state">Loading ValueGate…</div></div>'; return; }
    const data = this.data || { recent_assessments: [], aggregate: {} };
    const aggregate = data.aggregate || {}, tasks = Array.isArray(data.recent_assessments) ? data.recent_assessments : [];
    const rate = aggregate.total ? `${((aggregate.accepted || 0) / aggregate.total * 100).toFixed(1)}%` : '—';
    const rows = tasks.map((task) => `<tr><td>${this.escape(task.task_id)}</td><td>${this.money(task.cost_micros)}</td><td><span class="tag ${task.outcome_accepted ? 'tg' : 'tr'}">${task.outcome_accepted ? 'accepted' : 'rejected'}</span></td><td>${task.cpao_micros == null ? '—' : this.money(task.cpao_micros)}</td></tr>`).join('');
    this.innerHTML = `<div class="hero r4"><div class="hc"><div class="hl">Total tasks</div><div class="hv">${aggregate.total || 0}</div></div><div class="hc"><div class="hl">Accepted rate</div><div class="hv">${rate}</div></div><div class="hc"><div class="hl">Average CPAO</div><div class="hv">${aggregate.avg_cpao == null ? '—' : this.money(aggregate.avg_cpao)}</div></div><div class="hc"><div class="hl">Total cost</div><div class="hv">${this.money(aggregate.total_cost)}</div></div></div><div class="card"><h3>Recent task assessments</h3>${tasks.length ? `<div class="table-wrap"><table><thead><tr><th>Task</th><th>Cost</th><th>Outcome</th><th>CPAO</th></tr></thead><tbody>${rows}</tbody></table></div>` : '<div class="empty-state">No ValueGate assessments recorded yet.</div>'}</div>`;
  }
  escape(value) { const node = document.createElement('span'); node.textContent = String(value || ''); return node.innerHTML; }
}
customElements.define('cockpit-value-gate', CockpitValueGate);
