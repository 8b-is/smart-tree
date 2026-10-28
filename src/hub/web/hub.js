'use strict';
const $ = selector => document.querySelector(selector);
const node = (tag, text, className) => { const element = document.createElement(tag); if (text !== undefined) element.textContent = text; if (className) element.className = className; return element; };
const number = value => new Intl.NumberFormat().format(value);
const bytes = value => { if (!Number.isFinite(value)) return '—'; const units = ['B', 'KB', 'MB', 'GB', 'TB']; let unit = 0; while (value >= 1000 && unit < 4) { value /= 1000; unit++; } return `${value.toFixed(unit > 2 ? 1 : 0)} ${units[unit]}`; };
let toastTimer;
function toast(message) { const element = $('#toast'); if (!element) return; element.textContent = message; element.classList.add('show'); clearTimeout(toastTimer); toastTimer = setTimeout(() => element.classList.remove('show'), 2600); }
async function copy(text) { try { await navigator.clipboard.writeText(text); toast('Copied to clipboard'); } catch { toast('Clipboard unavailable. Select and copy the text.'); } }
function setTheme(theme) { document.documentElement.dataset.theme = theme; try { localStorage.setItem('theme', theme); } catch {} const button = $('#theme'); if (button) { button.textContent = theme === 'dark' ? '☼' : '◐'; button.setAttribute('aria-label', `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`); } }
let theme; try { theme = localStorage.getItem('theme'); } catch {} setTheme(theme || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'));
$('#theme')?.addEventListener('click', () => setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'));
document.querySelectorAll('[data-copy]').forEach(button => button.addEventListener('click', () => copy(document.getElementById(button.dataset.copy).textContent)));
document.querySelectorAll('[data-dialog]').forEach(button => button.addEventListener('click', () => document.getElementById(button.dataset.dialog).showModal()));
document.querySelectorAll('dialog .close').forEach(button => button.addEventListener('click', () => button.closest('dialog').close()));
document.querySelectorAll('dialog').forEach(dialog => dialog.addEventListener('click', event => { if (event.target === dialog) { const box = dialog.getBoundingClientRect(); if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) dialog.close(); } }));
const konami = ['ArrowUp','ArrowUp','ArrowDown','ArrowDown','ArrowLeft','ArrowRight','ArrowLeft','ArrowRight','b','a']; let konamiIndex = 0;
document.addEventListener('keydown', event => { if (/INPUT|TEXTAREA|SELECT/.test(event.target.tagName)) return; if (event.key === konami[konamiIndex]) { konamiIndex++; if (konamiIndex === konami.length) { document.documentElement.classList.toggle('cyberpunk'); toast('A little i1.is energy.'); konamiIndex = 0; } } else konamiIndex = 0; });

async function api(url, options = {}) {
  const response = await fetch(url, { ...options, headers: { ...(options.body ? {'Content-Type': 'application/json'} : {}), ...options.headers } });
  const data = await response.json().catch(() => ({ error: 'The hub returned an unreadable response.' }));
  if (!response.ok) throw new Error(data.error || `Request failed (${response.status})`);
  return data;
}
function externalLink(text, url) { const link = node('a', text); link.href = url; link.target = '_blank'; link.rel = 'noopener noreferrer'; return link; }
let offset = 0, loading = false;
async function loadRepositories(reset = false) {
  if (loading) return;
  loading = true;
  if (reset) offset = 0;
  const grid = $('#repositories');
  const query = new URLSearchParams({ collection: $('#collection').value, limit: '12', offset: String(offset) });
  try {
    const data = await api(`/api/v1/repositories?${query}`);
    if (reset) grid.replaceChildren();
    for (const repo of data.repositories) {
      const card = node('button', undefined, 'repo-card'); card.type = 'button';
      card.append(node('span', `⑂  ${repo.collection}`, 'repo-owner'), node('h3', repo.name));
      const meta = node('div', undefined, 'repo-meta');
      const label = repo.status === 'ready' ? 'Recall ready' : repo.status === 'indexing' ? 'Indexing' : repo.status === 'queued' ? 'Queued' : repo.status === 'archived' ? 'Archived' : repo.status === 'failed' ? 'Needs review' : 'Pending review';
      meta.append(node('span', label, `badge ${repo.status}`), node('span', `${number(repo.indexed_files)} files indexed`)); card.append(meta);
      card.addEventListener('click', () => showRepository(repo)); grid.append(card);
    }
    if (!grid.childElementCount) grid.append(node('p', 'This collection is being prepared. Check back as repositories join the archive.', 'empty'));
    offset += data.repositories.length; $('#load-more').hidden = !data.has_more;
  } catch (error) { if (reset) grid.replaceChildren(node('p', `Catalogue unavailable: ${error.message}`, 'empty')); toast(error.message); }
  finally { loading = false; }
}
function showRepository(repo) {
  $('#repo-title').textContent = repo.name;
  const details = $('#repo-details'); details.replaceChildren();
  details.append(node('p', `${repo.collection} · ${repo.status.replaceAll('_', ' ')}`));
  details.append(node('p', `${number(repo.indexed_files)} files / ${number(repo.indexed_passages)} passages indexed. ${number(repo.skipped_files)} files or passages skipped by coverage limits.`));
  if (repo.message) details.append(node('p', repo.message));
  if (repo.commit) details.append(node('p', `Commit ${repo.commit}`, 'small mono'));
  const command = `git clone https://8s.is${repo.clone_path}`;
  details.append(node('pre', command, 'repo-clone'));
  const button = node('button', 'Copy clone command', 'secondary'); button.addEventListener('click', () => copy(command)); details.append(button);
  const links = node('p'); links.append(externalLink('View original repository ↗', repo.source_url.replace(/\.git$/, ''))); details.append(links);
  if (repo.recall_opt_in && repo.status === 'ready') { const recallButton = node('button', 'Search this repository', 'primary'); recallButton.addEventListener('click', () => { $('#repo-dialog').close(); $('#search').dataset.repository = repo.id; $('#search').placeholder = `Search ${repo.name}`; $('#search').focus(); window.scrollTo({top:0, behavior:'smooth'}); }); details.append(recallButton); }
  $('#repo-dialog').showModal();
}
async function loadStats() {
  try {
    const data = await api('/api/v1/stats');
    $('#repo-count').textContent = number(data.repositories); $('#passage-count').textContent = number(data.indexed_passages);
    $('#storage-free').textContent = bytes(data.storage?.archives?.available_bytes);
    $('#connection').textContent = 'A HOME FOR USEFUL KNOWLEDGE'; $('#status-dot').classList.add('online');
    const select = $('#collection'), current = select.value; select.replaceChildren(new Option('All collections', ''));
    for (const collection of data.collections) select.add(new Option(`${collection.name} (${number(collection.repositories)})`, collection.name));
    select.value = current;
  } catch { $('#connection').textContent = 'The hub is temporarily unavailable'; $('#status-dot').classList.remove('online'); }
}
async function submitSearch(event) {
  event?.preventDefault(); const input = $('#search'); const query = input.value.trim(); if (!query) { input.focus(); return; }
  const button = $('#search-button'); button.disabled = true; button.textContent = 'Finding…';
  $('#results-section').hidden = false; $('#result-summary').textContent = 'Looking through the saved index…'; $('#results').replaceChildren();
  try {
    const data = await api('/api/v1/recall', {method:'POST', body:JSON.stringify({query, collection:$('#collection').value, repository:input.dataset.repository || '', limit:8})});
    $('#result-summary').textContent = `${data.mode === 'hybrid' ? 'Semantic + keyword recall' : 'Keyword recall'} · ${data.results.length} passages · ${number(data.elapsed_ms)} ms`;
    for (const result of data.results) {
      const article = node('article', undefined, 'result'); article.append(node('span', result.repository, 'repo-owner muted'));
      const heading = node('h3'); heading.append(externalLink(result.path, result.source_url)); article.append(heading);
      article.append(node('div', `Lines ${result.line_start}–${result.line_end} · commit ${result.commit.slice(0,12)}`, 'result-meta'), node('pre', result.text));
      $('#results').append(article);
    }
    if (!data.results.length) $('#results').append(node('p', 'No matching passages yet. Try a different phrase or a collection that has finished indexing.', 'empty'));
    $('#results-section').scrollIntoView({behavior:'smooth', block:'start'});
  } catch (error) { $('#result-summary').textContent = error.message; }
  finally { button.disabled = false; button.textContent = 'Recall ↗'; }
}
let receipt;
async function handleForm(form, action) {
  const button = form.querySelector('button[type=submit]'); const status = form.querySelector('.form-status');
  button.disabled = true; status.textContent = 'Saving…';
  try { await action(status); } catch (error) { status.textContent = error.message; } finally { button.disabled = false; }
}
if ($('#search-form')) {
  $('#search-form').addEventListener('submit', submitSearch);
  document.querySelectorAll('[data-query]').forEach(button => button.addEventListener('click', () => { $('#search').value = button.dataset.query; delete $('#search').dataset.repository; submitSearch(); }));
  $('#clear-results').addEventListener('click', () => { $('#results-section').hidden = true; delete $('#search').dataset.repository; $('#search').placeholder = 'What would you like to find?'; $('#search').focus(); });
  $('#collection').addEventListener('change', () => { delete $('#search').dataset.repository; loadRepositories(true); });
  $('#load-more').addEventListener('click', () => loadRepositories());
  $('#archive-form').addEventListener('submit', event => { event.preventDefault(); handleForm(event.target, async status => {
    const result = await api('/api/v1/archive-requests', {method:'POST', body:JSON.stringify({source_url:$('#archive-url').value, public:$('#archive-public').checked, recall_opt_in:$('#archive-recall').checked})});
    receipt = {hub:'https://8s.is', repository_id:result.repository.id, manage_token:result.manage_token, source_url:result.repository.source_url};
    status.textContent = 'Request saved. It is awaiting operator review.'; $('#receipt').hidden = false; $('#receipt-id').textContent = `Archive ID: ${result.repository.id}`;
    $('#manage-id').value = receipt.repository_id; $('#manage-token').value = receipt.manage_token;
  }); });
  $('#save-receipt').addEventListener('click', () => { if (!receipt) return; const url = URL.createObjectURL(new Blob([JSON.stringify(receipt,null,2)], {type:'application/json'})); const link = node('a'); link.href = url; link.download = `smart-tree-archive-${receipt.repository_id}.json`; link.click(); setTimeout(() => URL.revokeObjectURL(url),1000); });
  $('#feedback-form').addEventListener('submit', event => { event.preventDefault(); handleForm(event.target, async status => {
    const result = await api('/api/feedback', {method:'POST', body:JSON.stringify({category:$('#feedback-category').value, title:$('#feedback-title').value, description:$('#feedback-description').value, impact_score:5, frequency_score:5, anonymous:true, source:'hub-website'})});
    status.textContent = `Thank you. Feedback saved: ${result.feedback_id}`;
  }); });
  $('#manage-form').addEventListener('submit', event => { event.preventDefault(); handleForm(event.target, async status => {
    const repo = await api(`/api/v1/repositories/${encodeURIComponent($('#manage-id').value.trim())}`, {headers:{Authorization:`Bearer ${$('#manage-token').value.trim()}`}});
    $('#manage-name').textContent = `${repo.collection}/${repo.name}`; $('#manage-state').textContent = `${repo.status.replaceAll('_',' ')} · ${repo.message}`;
    $('#manage-public').checked = repo.public; $('#manage-recall').checked = repo.recall_opt_in; $('#manage-details').hidden = false; status.textContent = '';
  }); });
  $('#save-permissions').addEventListener('click', async () => {
    const button = $('#save-permissions'); button.disabled = true; $('#manage-status').textContent = 'Saving…';
    try { const result = await api(`/api/v1/repositories/${encodeURIComponent($('#manage-id').value.trim())}`, {method:'PATCH', headers:{Authorization:`Bearer ${$('#manage-token').value.trim()}`}, body:JSON.stringify({public:$('#manage-public').checked, recall_opt_in:$('#manage-recall').checked})}); $('#manage-status').textContent = 'Permissions saved.'; $('#manage-state').textContent = result.status.replaceAll('_',' '); }
    catch (error) { $('#manage-status').textContent = error.message; } finally { button.disabled = false; }
  });
  loadStats(); loadRepositories(true);
  setInterval(() => { if (!document.hidden) loadStats(); },30000);
}
