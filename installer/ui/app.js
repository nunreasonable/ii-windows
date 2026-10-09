"use strict";
(() => {
	const tauri = window.__TAURI__;
	const invoke = (cmd, args) => tauri.core.invoke(cmd, args);
	const listen = (ev, cb) => tauri.event.listen(ev, cb);

	const ICONS = {
		install: '<path d="M12 4v11M7.5 10.5 12 15l4.5-4.5M5 19.5h14"/>',
		update: '<path d="M19.5 10.5A7.5 7.5 0 0 0 6 7M4.5 3.5V7.5h4"/><path d="M4.5 13.5A7.5 7.5 0 0 0 18 17M19.5 20.5v-4h-4"/>',
		repair: '<path d="M14.7 6.3a4.2 4.2 0 0 0-5.5 5.2l-5 5a1.9 1.9 0 0 0 2.7 2.7l5-5a4.2 4.2 0 0 0 5.2-5.5l-2.6 2.6-2.4-.4-.4-2.4z"/>',
		uninstall: '<path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 12.5h9l1-12.5M10 11v5M14 11v5"/>',
		check: '<path d="M5 12.5l4.5 4.5L19 7.5"/>',
		arrow: '<path d="M5 12h14M13 6l6 6-6 6"/>',
		doc: '<path d="M7 3.5h7l4.5 4.5v12.5h-11.5z"/><path d="M14 3.5V8h4.5M10 12.5h5M10 16h5"/>',
		terminal: '<rect x="3" y="4.5" width="18" height="15" rx="2.5"/><path d="M7 9.5l3 2.5-3 2.5M12.5 15h4.5"/>',
		power: '<path d="M12 3.5v8"/><path d="M6.6 6.8a7.5 7.5 0 1 0 10.8 0"/>',
		shield: '<path d="M12 3.5l7.5 3v5.5c0 4.6-3.2 7.6-7.5 8.5-4.3-.9-7.5-3.9-7.5-8.5V6.5z"/><path d="M12 8.5v4M12 15.5v.01"/>',
		pwsh: '<rect x="3" y="4.5" width="18" height="15" rx="2.5"/><path d="M7.5 9.5l3.5 2.5-3.5 2.5M13 15h4"/>',
		play: '<path class="fill" d="M8 5.5v13l10.5-6.5z"/>',
		box: '<path d="M12 3.5l7.5 4.2v8.6L12 20.5l-7.5-4.2V7.7z"/><path d="M4.5 7.7 12 12l7.5-4.3M12 12v8.5"/>',
		palette: '<path d="M12 3.5a8.5 8.5 0 1 0 0 17c1.1 0 1.6-.8 1.6-1.6 0-1.2-1-1.5-1-2.6 0-1 .8-1.6 1.8-1.6h1.9a4.2 4.2 0 0 0 4.2-4.2c0-3.9-3.8-7-8.5-7z"/><circle class="fill" cx="7.5" cy="12" r="1.2"/><circle class="fill" cx="9.5" cy="7.8" r="1.2"/><circle class="fill" cx="14.3" cy="7.8" r="1.2"/>',
		sliders: '<path d="M4 7h9M17 7h3M4 17h3M11 17h9"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="17" r="2"/>',
		info: '<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5M12 7.8v.01"/>',
		warn: '<path d="M12 4 21 19.5H3z"/><path d="M12 10v4.5M12 17v.01"/>',
		error: '<circle cx="12" cy="12" r="8.5"/><path d="M9 9l6 6M15 9l-6 6"/>',
		ok: '<circle cx="12" cy="12" r="8.5"/><path d="M8.5 12.3l2.4 2.4 4.6-4.9"/>',
		skip: '<path d="M7 12h10"/>',
		close: '<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>',
		expand: '<path d="M9 4.5H4.5V9M15 4.5h4.5V9M9 19.5H4.5V15M15 19.5h4.5V15"/>',
		folder: '<path d="M3.5 7A1.5 1.5 0 0 1 5 5.5h4l2 2h8A1.5 1.5 0 0 1 20.5 9v8.5A1.5 1.5 0 0 1 19 19H5a1.5 1.5 0 0 1-1.5-1.5z"/>',
		rocket: '<path d="M12 15.5 8.5 12c.8-4 3.5-7.5 9-8.5-1 5.5-4.5 8.2-8.5 9"/><path d="M8.5 12H5l2.5-3.5h3.5M12 15.5V19l3.5-2.5V13"/>',
	};
	const icon = (name, cls = "") => `<svg viewBox="0 0 24 24" class="${cls}">${ICONS[name] || ""}</svg>`;
	const $ = (sel, root = document) => root.querySelector(sel);
	const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];
	const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
	const mb = (bytes) => (bytes >= 1073741824 ? `${(bytes / 1073741824).toFixed(1)} GB` : `${Math.max(1, Math.round(bytes / 1048576))} MB`);

	let lang = "en";
	let info = null;
	const S = {
		page: "welcome",
		action: null,
		readEnd: false,
		readChecked: false,
		sources: {},
		preflight: {},
		opts: null,
		uopts: null,
		force: false,
		launch: true,
		installWinget: true,
		plan: [],
		steps: {},
		result: null,
		logOpen: false,
		runningAction: null,
	};

	function t(key, params) {
		let s = (I18N[lang] && I18N[lang][key]) ?? I18N.en[key] ?? key;
		if (params) for (const [k, v] of Object.entries(params)) s = s.split(`{${k}}`).join(v);
		return s;
	}
	function msg(m) {
		if (!m) return "";
		const k = `m_${m.key}`;
		if ((I18N[lang] && I18N[lang][k]) || I18N.en[k]) return t(k, m.params);
		return m.text;
	}

	function versionCmp(a, b) {
		const p = (v) => String(v || "").replace(/^v/i, "").split("-")[0].split(".").map((x) => parseInt(x, 10) || 0);
		const x = p(a), y = p(b);
		for (let i = 0; i < Math.max(x.length, y.length); i++) {
			const d = (x[i] || 0) - (y[i] || 0);
			if (d) return d;
		}
		return 0;
	}

	let toastTimer = 0;
	function toast(text) {
		const el = $("#toast");
		el.textContent = text;
		el.classList.add("show");
		clearTimeout(toastTimer);
		toastTimer = setTimeout(() => el.classList.remove("show"), 2600);
	}

	const ORDER = ["welcome", "readme", "options", "progress", "done"];

	function showPage(name) {
		const from = ORDER.indexOf(S.page);
		const to = ORDER.indexOf(name);
		const back = to < from;
		for (const p of $$(".page")) {
			p.classList.remove("leaving", "leaving-back");
			if (p.dataset.page === name) {
				p.classList.add("active");
			} else if (p.classList.contains("active")) {
				p.classList.remove("active");
				p.classList.add(back ? "leaving-back" : "leaving");
			}
		}
		S.page = name;
		$$("#stepper li").forEach((li) => {
			const i = ORDER.indexOf(li.dataset.step);
			li.classList.toggle("current", i === to);
			li.classList.toggle("past", i < to);
		});
		if (name === "readme") requestAnimationFrame(updateReadProgress);
		if (name === "readme") setTimeout(() => $("#readme").focus({ preventScroll: true }), 350);
	}

	function applyStatic() {
		document.documentElement.lang = lang === "pt" ? "pt-BR" : "en";
		$$("[data-i18n]").forEach((el) => (el.textContent = t(el.dataset.i18n)));
		$$("[data-icon]").forEach((el) => (el.innerHTML = icon(el.dataset.icon)));
		$$("#stepper .dot").forEach((d) => (d.innerHTML = icon("check")));
		$$(".lang-toggle button").forEach((b) => b.classList.toggle("on", b.dataset.lang === lang));
		$("#setup-version").textContent = t("setup_version", { v: info ? info.setup_version : "" });
		const releaseName = $("#release-name");
		releaseName.textContent = info && info.release_name ? `${info.setup_version} · ${info.release_name}` : "";
		releaseName.hidden = !(info && info.release_name);
		$("#toggle-log span:last-child").textContent = t(S.logOpen ? "hide_details" : "show_details");
	}

	function setLang(l) {
		lang = l;
		applyStatic();
		renderWelcome();
		renderReadme();
		if (S.page === "options") renderOptions();
		if (S.page === "progress" || S.page === "done") renderProgressHead();
		renderSteps();
		if (S.page === "done") renderDone();
	}

	function actionAllowed(a) {
		if (a === "install") return true;
		return !!info.installed;
	}

	function renderWelcome() {
		const chips = [];
		if (info.installed) chips.push(`<span class="chip ok">${icon("ok")}${esc(t("chip_installed", { v: info.installed.version }))}</span>`);
		else chips.push(`<span class="chip">${esc(t("chip_not_installed"))}</span>`);
		if (info.offline) chips.push(`<span class="chip primary">${icon("box")}${esc(t("chip_offline", { v: info.offline.version }))}</span>`);
		if (info.offline_error) chips.push(`<span class="chip error" title="${esc(info.offline_error)}">${icon("error")}${esc(t("chip_offline_error"))}</span>`);
		if (info.unmanaged_files) chips.push(`<span class="chip warn">${icon("warn")}${esc(t("chip_unmanaged"))}</span>`);
		if (info.manifest_error) chips.push(`<span class="chip error" title="${esc(info.manifest_error)}">${icon("error")}${esc(t("chip_manifest_error"))}</span>`);
		$("#welcome-chips").innerHTML = chips.join("");

		for (const card of $$(".action")) {
			const a = card.dataset.action;
			const reinstall = a === "install" && info.installed;
			$(".action-title", card).textContent = t(reinstall ? "act_reinstall" : `act_${a}`);
			const allowed = actionAllowed(a);
			$(".action-desc", card).textContent = allowed ? t(reinstall ? "act_reinstall_desc" : `act_${a}_desc`) : t("need_install");
			card.disabled = !allowed;
			card.classList.toggle("selected", S.action === a);
		}
		$("#welcome-next").disabled = !S.action;
		$("#welcome-note").textContent = info.offline_error || "";
	}

	function selectAction(a) {
		if (!actionAllowed(a)) return;
		if (S.action !== a) {
			S.action = a;
			S.opts = null;
			S.uopts = null;
			S.force = false;
			S.launch = true;
			S.installWinget = true;
		}
		renderWelcome();
	}

	function renderReadme() {
		const el = $("#readme");
		const top = el.scrollTop;
		el.innerHTML = lang === "pt" ? info.readme_pt : info.readme_en;
		el.scrollTop = top;
		updateReadProgress();
	}

	function updateReadProgress() {
		const el = $("#readme");
		const max = el.scrollHeight - el.clientHeight;
		const frac = max <= 8 ? 1 : Math.min(1, el.scrollTop / max);
		$("#read-bar").style.width = `${Math.round(frac * 100)}%`;
		const atEnd = max <= 8 || el.scrollTop >= max - 8;
		$(".readme-wrap").classList.toggle("at-end", atEnd);
		if (atEnd && !S.readEnd && S.page === "readme") {
			S.readEnd = true;
			$("#read-check-label").classList.add("ready");
		}
		const box = $("#read-check");
		box.disabled = !S.readEnd;
		box.checked = S.readChecked && S.readEnd;
		$("#read-check-label").classList.toggle("disabled", !S.readEnd);
		$("#read-hint").textContent = t(S.readEnd ? "read_hint_done" : "read_hint");
		$("#readme-next").disabled = !(S.readEnd && S.readChecked);
	}

	const STYLES = [
		{ id: "ii", img: "img/style-ii.webp" },
		{ id: "end4pc", img: "img/style-end4pc.webp" },
	];

	function defaultOpts() {
		const o = info.installed ? { ...info.installed.options } : { autostart: true, terminal: true, pwsh7: false, exec_policy: false, ffmpeg: false };
		o.visual_style = info.current_style || (STYLES.some((s) => s.id === o.visual_style) ? o.visual_style : "ii");
		return o;
	}

	function styleCard(o) {
		const items = STYLES.map((s) => {
			const on = o.visual_style === s.id;
			return `<button type="button" class="style-opt${on ? " selected" : ""}" data-style="${s.id}" role="radio" aria-checked="${on}">
				<span class="style-shot"><img src="${s.img}" alt="${esc(t(`style_${s.id}`))}" draggable="false"><span class="style-zoom" data-zoom="${s.img}" title="${esc(t("style_zoom"))}">${icon("expand")}</span></span>
				<span class="style-name">${esc(t(`style_${s.id}`))}<span class="style-check">${icon("check")}</span></span>
				<span class="style-sub">${esc(t(`style_${s.id}_desc`))}</span>
			</button>`;
		}).join("");
		return `<div class="card"><div class="card-title">${esc(t("card_style"))}</div><div class="style-grid" role="radiogroup">${items}</div><div class="style-note">${esc(t("style_note"))}</div></div>`;
	}

	function openZoom(src) {
		const box = $("#lightbox");
		box.querySelector("img").src = src;
		box.hidden = false;
	}

	function switchRow(id, iconName, title, desc, checked, { disabled = false, extra = "", extraCls = "info", caution = false } = {}) {
		return `<label class="row switch-row${disabled ? " disabled" : ""}${caution ? " caution" : ""}" data-switch="${id}">
			<span class="row-icon">${icon(iconName)}</span>
			<span class="row-text"><span class="row-title">${esc(title)}</span><span class="row-desc">${desc}</span>${extra ? `<span class="row-extra ${extraCls}">${extra}</span>` : ""}</span>
			<span class="switch"><input type="checkbox" ${checked ? "checked" : ""} ${disabled ? "disabled" : ""}><span class="track"></span><span class="thumb"></span></span>
		</label>`;
	}

	function sourceCard(action) {
		const src = S.sources[action];
		let body;
		if (!src || src.loading) {
			const v = action === "repair" && info.installed ? info.installed.version : null;
			body = `<div class="source"><span class="spinner"></span><span class="loading">${esc(v ? t("finding_version", { v }) : t("finding_release"))}</span></div>`;
		} else if (src.error) {
			body = `<div class="source error"><span class="row-icon">${icon("error")}</span><span class="row-text"><span class="row-title">${esc(t("pkg_error_title"))}</span><span class="row-desc">${esc(msg(src.error))}</span></span><button type="button" class="mini-btn" data-retry>${esc(t("retry"))}</button></div>`;
		} else {
			const s = src.value;
			const line = s.kind === "offline" ? t("pkg_offline", { path: s.path }) : t("pkg_github", { tag: s.release.tag, size: mb(s.release.zip_size) });
			const ver = s.kind === "offline" ? s.version : s.release.version;
			let title = `ii-windows ${esc(ver)}`;
			if (action === "update" && info.installed) {
				title = `<span class="version-flow"><span class="from">${esc(info.installed.version)}</span>${icon("arrow")}<span>${esc(ver)}</span></span>`;
			}
			body = `<div class="source"><span class="row-icon">${icon("box")}</span><span class="row-text"><span class="row-title">${title}</span><span class="row-desc">${esc(line)}</span></span></div>`;
		}
		return `<div class="card"><div class="card-title">${esc(t("card_package"))}</div>${body}</div>`;
	}

	function checksCard(action) {
		const pf = S.preflight[action];
		let body;
		if (!pf || pf.loading) body = `<div class="checks"><div class="check-item"><span class="spinner"></span>${esc(t("checking_system"))}</div></div>`;
		else
			body = `<div class="checks">${pf.value.checks
				.map((c) => `<div class="check-item ${c.level}">${icon(c.level === "ok" ? "ok" : c.level === "info" ? "info" : c.level === "warn" ? "warn" : "error")}<span>${esc(msg(c.msg))}</span></div>`)
				.join("")}</div>`;
		return `<div class="card"><div class="card-title">${esc(t("card_checks"))}</div>${body}</div>`;
	}

	function sourceVersion(action) {
		const src = S.sources[action];
		if (!src || !src.value) return null;
		return src.value.kind === "offline" ? src.value.version : src.value.release.version;
	}

	function renderOptions() {
		const a = S.action;
		const head = $("#options-icon");
		head.innerHTML = icon(a);
		head.classList.toggle("danger", a === "uninstall");
		const reinstall = a === "install" && info.installed;
		$("#options-title").textContent = t(reinstall ? "opt_reinstall_title" : `opt_${a}_title`);
		$("#options-sub").textContent = t(a === "install" ? "opt_install_sub" : `opt_${a}_sub`);
		const pf = S.preflight[a] && S.preflight[a].value;
		let html = "";

		if (a === "install") {
			if (!S.opts) S.opts = defaultOpts();
			const o = S.opts;
			html += sourceCard(a);
			html += styleCard(o);
			const pwshPresent = pf && pf.pwsh;
			const policyEff = pf && pf.exec_policy_effective;
			const policyAllowed = policyEff && /^(RemoteSigned|Unrestricted|Bypass)$/i.test(policyEff);
			const policyOurs = info.installed && info.installed.exec_policy_changed;
			let policyExtra = "", policyCls = "info", policyDisabled = false;
			if (policyOurs) { policyExtra = esc(t("o_policy_ours", { p: info.installed.exec_policy_previous || "Undefined" })); }
			else if (policyAllowed) { policyExtra = esc(t("o_policy_allowed", { p: policyEff })); policyCls = "ok"; policyDisabled = true; }
			else if (policyEff) { policyExtra = esc(t("o_policy_now", { p: `${pf.exec_policy_current_user || "Undefined"} (${policyEff})` })); policyCls = "warn"; }
			if (policyDisabled) o.exec_policy = false;
			if (pwshPresent && !(info.installed && info.installed.pwsh7_ours)) o.pwsh7 = false;
			const ffmpegPresent = pf && pf.ffmpeg_present;
			if (ffmpegPresent && !(info.installed && info.installed.ffmpeg_ours)) o.ffmpeg = false;
			const win10NoWinget = !!(pf && !pf.winget && pf.windows_build >= 19041 && pf.windows_build < 22000);
			if (!win10NoWinget) S.installWinget = true;
			let terminalExtra = "", terminalExtraCls = "warn";
			if (pf && !pf.winget) {
				if (!(win10NoWinget && S.installWinget)) terminalExtra = esc(t("o_terminal_nowinget"));
			} else if (pf && !pf.windows_terminal_present) {
				terminalExtra = esc(t("o_terminal_wt"));
				terminalExtraCls = "info";
			}
			let ffmpegExtra = "", ffmpegExtraCls = "warn";
			if (pf && !pf.winget && !(win10NoWinget && S.installWinget)) ffmpegExtra = esc(t("o_ffmpeg_nowinget"));
			html += `<div class="card"><div class="card-title">${esc(t("card_options"))}</div>
				${switchRow("autostart", "power", t("o_autostart"), esc(t("o_autostart_desc")), o.autostart)}
				${switchRow("terminal", "terminal", t("o_terminal"), esc(t("o_terminal_desc")), o.terminal, { extra: terminalExtra, extraCls: terminalExtraCls })}
				${win10NoWinget && (o.terminal || o.ffmpeg) ? switchRow("install_winget", "box", t("o_install_winget"), esc(t("o_install_winget_desc")), S.installWinget) : ""}
				${switchRow("pwsh7", "pwsh", t("o_pwsh7"), esc(t("o_pwsh7_desc")), o.pwsh7, { disabled: !!pwshPresent, extra: pwshPresent ? esc(t("o_pwsh7_present")) : "", extraCls: "ok" })}
				${switchRow("ffmpeg", "box", t("o_ffmpeg"), esc(t("o_ffmpeg_desc")), o.ffmpeg, { disabled: !!ffmpegPresent, extra: ffmpegPresent ? esc(t("o_ffmpeg_present")) : ffmpegExtra, extraCls: ffmpegPresent ? "ok" : ffmpegExtraCls })}
				${switchRow("exec_policy", "shield", t("o_policy"), esc(t("o_policy_desc")).replace("Set-ExecutionPolicy -Scope CurrentUser RemoteSigned", "<code>Set-ExecutionPolicy -Scope CurrentUser RemoteSigned</code>"), o.exec_policy, { disabled: policyDisabled, extra: policyExtra, extraCls: policyCls, caution: true })}
				${switchRow("launch", "rocket", t("o_launch"), esc(t("o_launch_desc")), S.launch)}
			</div>`;
			html += checksCard(a);
		} else if (a === "update") {
			if (!S.opts) S.opts = defaultOpts();
			html += sourceCard(a);
			const v = sourceVersion(a);
			if (v) {
				const newer = versionCmp(v, info.installed.version) > 0;
				html += `<div class="card"><div class="card-title">${esc(newer ? t("update_available") : t("update_none"))}</div>
					${newer ? "" : switchRow("force", "update", t("force_title"), esc(t("force_desc", { v })), S.force)}
					${switchRow("launch", "rocket", t("o_restart"), esc(t("o_launch_desc")), S.launch)}
				</div>`;
			}
			html += styleCard(S.opts);
			html += checksCard(a);
		} else if (a === "repair") {
			html += sourceCard(a);
			html += `<div class="card"><div class="card-title">${esc(t("card_what"))}</div><ul class="bullets">
				<li>${esc(t("r_b1", { p: info.settings_dir }))}</li>
				<li>${esc(t("r_b2", { v: info.installed.version }))}</li>
				<li>${esc(t("r_b3"))}</li>
				<li>${esc(t("r_b4"))}</li>
			</ul>${switchRow("launch", "rocket", t("o_restart"), esc(t("o_launch_desc")), S.launch)}</div>`;
			html += checksCard(a);
		} else if (a === "uninstall") {
			const inst = info.installed;
			if (!S.uopts) S.uopts = { remove_tools: inst.tools.length > 0, remove_pwsh7: inst.pwsh7_ours, remove_ffmpeg: inst.ffmpeg_ours, keep_settings: false, restore_look: inst.has_pre_install };
			const u = S.uopts;
			html += `<div class="card"><div class="card-title">${esc(t("card_options"))}</div>
				${switchRow("remove_tools", "terminal", t("u_tools"), esc(inst.tools.length ? inst.tools.join(", ") : t("u_tools_none")), u.remove_tools, { disabled: !inst.tools.length })}
				${inst.pwsh7_ours ? switchRow("remove_pwsh7", "pwsh", t("u_pwsh7"), esc(t("u_pwsh7_desc")), u.remove_pwsh7) : ""}
				${inst.ffmpeg_ours ? switchRow("remove_ffmpeg", "box", t("u_ffmpeg"), esc(t("u_ffmpeg_desc")), u.remove_ffmpeg) : ""}
				${switchRow("keep_settings", "sliders", t("u_keep"), esc(t("u_keep_desc", { p: info.settings_dir })), u.keep_settings)}
				${switchRow("restore_look", "palette", t("u_restore"), esc(inst.has_pre_install ? t("u_restore_desc") : t("u_restore_none")), u.restore_look, { disabled: !inst.has_pre_install })}
			</div>`;
			html += checksCard(a);
		}
		const body = $("#options-body");
		const top = body.scrollTop;
		body.innerHTML = html;
		body.scrollTop = top;
		updateGo();
	}

	function updateGo() {
		const a = S.action;
		const go = $("#options-go");
		const reinstall = a === "install" && info.installed;
		go.textContent = t(reinstall ? "go_reinstall" : `go_${a}`);
		go.classList.toggle("danger", a === "uninstall");
		const pf = S.preflight[a];
		const src = S.sources[a];
		let ok = pf && pf.value && !pf.value.blocked;
		if (a !== "uninstall") ok = ok && src && src.value;
		if (a === "update" && ok) ok = versionCmp(sourceVersion(a), info.installed.version) > 0 || S.force;
		go.disabled = !ok;
		$("#options-note").textContent = pf && pf.value && pf.value.blocked ? t("blocked") : "";
	}

	function onSwitch(id, checked) {
		if (id === "launch") S.launch = checked;
		else if (id === "force") S.force = checked;
		else if (id === "install_winget") S.installWinget = checked;
		else if (S.action === "uninstall") S.uopts[id] = checked;
		else S.opts[id] = checked;
		if (id === "terminal" || id === "ffmpeg" || id === "force" || id === "install_winget") renderOptions();
		else updateGo();
	}

	async function loadSource(action) {
		S.sources[action] = { loading: true };
		if (S.page === "options") renderOptions();
		try {
			const value = await invoke("source", { action });
			S.sources[action] = { value };
		} catch (error) {
			S.sources[action] = { error: typeof error === "object" ? error : { key: "internal", text: String(error), params: { error: String(error) } } };
		}
		if (S.page === "options" && S.action === action) renderOptions();
		return S.sources[action];
	}

	async function loadPreflight(action) {
		S.preflight[action] = { loading: true };
		const src = S.sources[action];
		let size = 0;
		if (src && src.value) size = src.value.kind === "offline" ? src.value.size : src.value.release.zip_size;
		const neededMb = Math.ceil((size || 100 * 1048576) * 2.6 / 1048576) + 64;
		const terminal = action === "install" ? (S.opts || defaultOpts()).terminal : !!(info.installed && info.installed.options.terminal);
		const ffmpeg = action === "install" ? (S.opts || defaultOpts()).ffmpeg : !!(info.installed && info.installed.options.ffmpeg);
		try {
			const value = await invoke("preflight", { action, terminal, ffmpeg, neededMb });
			S.preflight[action] = { value };
		} catch (e) {
			S.preflight[action] = { value: { checks: [{ id: "internal", level: "error", msg: { key: "internal", text: String(e), params: { error: String(e) } } }], blocked: true } };
		}
		if (S.page === "options" && S.action === action) renderOptions();
	}

	async function enterOptions() {
		const a = S.action;
		showPage("options");
		renderOptions();
		if (a !== "uninstall" && !(S.sources[a] && S.sources[a].value)) await loadSource(a);
		await loadPreflight(a);
	}

	function renderProgressHead() {
		const a = S.runningAction || S.action;
		if (!a) return;
		$("#progress-title").textContent = t(`prog_${a}`);
		$("#progress-sub").textContent = t("prog_sub");
	}

	function renderSteps() {
		const ol = $("#steps");
		if (!S.plan.length) { ol.innerHTML = ""; return; }
		ol.innerHTML = S.plan.map((id) => stepHtml(id)).join("");
		updateOverall();
	}

	function stepHtml(id) {
		const st = S.steps[id] || { status: "pending" };
		let ic = "";
		if (st.status === "running") ic = '<span class="spinner"></span>';
		else if (st.status === "done") ic = icon("check");
		else if (st.status === "skipped") ic = icon("skip");
		else if (st.status === "warning") ic = icon("warn");
		else if (st.status === "failed") ic = icon("close");
		const detail = st.detail ? `<div class="step-detail">${esc(msg(st.detail))}</div>` : "";
		const bar = st.status === "running" && st.fraction != null
			? `<div class="step-bar"><div style="width:${Math.round(st.fraction * 100)}%"></div></div>${st.text ? `<div class="step-progress-text">${esc(st.text)}</div>` : ""}`
			: "";
		return `<li class="step ${st.status}" data-step="${id}"><span class="step-icon">${ic}</span><div class="step-body"><div class="step-label">${esc(t(`s_${id}`))}</div>${detail}${bar}</div></li>`;
	}

	function updateStep(id) {
		const li = $(`#steps li[data-step="${id}"]`);
		if (li) li.outerHTML = stepHtml(id);
		else renderSteps();
		updateOverall();
	}

	function updateOverall() {
		if (!S.plan.length) return;
		let done = 0;
		let current = null;
		for (const id of S.plan) {
			const st = S.steps[id];
			if (!st) continue;
			if (["done", "skipped", "warning", "failed"].includes(st.status)) done += 1;
			else if (st.status === "running") { done += st.fraction || 0; current = id; }
		}
		const frac = S.result ? 1 : done / S.plan.length;
		$("#overall-bar").style.width = `${Math.round(frac * 100)}%`;
		$(".overall").classList.toggle("idle", !!S.result);
		const cancellable = !S.result && S.runningAction !== "uninstall" && ["download", "verify", "files", "winget"].includes(current);
		$("#cancel-btn").hidden = !cancellable;
		if (current) {
			const li = $(`#steps li[data-step="${current}"]`);
			if (li && !S.logOpen) li.scrollIntoView({ block: "nearest", behavior: "smooth" });
		}
	}

	const logLines = [];
	function appendLog(line) {
		logLines.push(line);
		if (logLines.length > 3000) logLines.splice(0, logLines.length - 3000);
		if (S.logOpen) {
			const el = $("#log");
			el.textContent += (el.textContent ? "\n" : "") + line;
			const sc = $("#progress-scroll");
			sc.scrollTop = sc.scrollHeight;
		}
	}

	function onEvent(e) {
		switch (e.type) {
			case "plan":
				S.plan = e.steps;
				S.steps = {};
				renderSteps();
				break;
			case "step": {
				const prev = S.steps[e.id] || {};
				S.steps[e.id] = { status: e.status, detail: e.detail || (e.status === "running" ? null : prev.detail), fraction: e.status === "running" ? prev.fraction : null };
				if (!S.plan.includes(e.id)) S.plan.push(e.id);
				updateStep(e.id);
				break;
			}
			case "progress": {
				const st = S.steps[e.id] || { status: "running" };
				st.fraction = e.fraction;
				st.text = e.text;
				S.steps[e.id] = st;
				const li = $(`#steps li[data-step="${e.id}"]`);
				const bar = li && $(".step-bar div", li);
				if (bar) {
					bar.style.width = `${Math.round(e.fraction * 100)}%`;
					const tx = $(".step-progress-text", li);
					if (tx && e.text) tx.textContent = e.text;
					updateOverall();
				} else updateStep(e.id);
				break;
			}
			case "log":
				appendLog(e.line);
				break;
			case "finished":
				S.result = e;
				for (const id of S.plan) {
					const st = S.steps[id];
					if (st && st.status === "running") st.status = e.ok ? "done" : "failed";
				}
				renderSteps();
				setTimeout(() => { renderDone(); showPage("done"); }, 700);
				break;
		}
	}

	async function start() {
		const a = S.action;
		const opts = {
			options: S.opts || defaultOpts(),
			launch: S.launch,
			install_winget: S.installWinget !== false,
			remove_tools: !!(S.uopts && S.uopts.remove_tools),
			remove_pwsh7: !!(S.uopts && S.uopts.remove_pwsh7),
			remove_ffmpeg: !!(S.uopts && S.uopts.remove_ffmpeg),
			keep_settings: !!(S.uopts && S.uopts.keep_settings),
			restore_look: !!(S.uopts && S.uopts.restore_look),
			force: S.force,
		};
		S.plan = [];
		S.steps = {};
		S.result = null;
		S.runningAction = a;
		logLines.length = 0;
		$("#log").textContent = "";
		renderSteps();
		renderProgressHead();
		$("#progress-icon").innerHTML = '<span class="spinner big"></span>';
		showPage("progress");
		try {
			await invoke("start", { action: a, opts });
		} catch (err) {
			onEvent({ type: "finished", ok: false, error: { key: "internal", text: String(err), params: { error: String(err) } }, warnings: [], notes: [], can_launch: false });
		}
	}

	function noteHtml(m, kind) {
		const path = m.params && m.params.path;
		const btn = path && kind === "note" ? `<button type="button" class="mini-btn" data-open="${esc(path)}">${icon("folder")}${esc(t("open"))}</button>` : "";
		return `<div class="${kind === "note" ? "note-item" : kind === "error" ? "warn-item error-item" : "warn-item"}">${icon(kind === "note" ? "info" : kind === "error" ? "error" : "warn")}<span class="note-text">${esc(msg(m))}</span>${btn}</div>`;
	}

	function renderDone() {
		const r = S.result;
		if (!r) return;
		const a = S.runningAction;
		const badge = $("#done-badge");
		const nWarn = r.warnings.length;
		const cancelled = !r.ok && r.error && r.error.key === "cancelled";
		badge.className = `done-badge${r.ok ? (nWarn ? " warn" : "") : cancelled ? " warn" : " fail"}`;
		badge.innerHTML = icon(r.ok ? (nWarn ? "warn" : "check") : cancelled ? "skip" : "close");
		$("#done-title").textContent = r.ok ? t(`done_${a}`) : cancelled ? t("done_cancelled") : t("done_fail");
		let sub = r.ok ? (nWarn ? t("done_warn_sub", { n: nWarn }) : t("done_ok_sub")) : cancelled ? msg(r.error) : t("done_fail_sub");
		const fin = S.steps.finish;
		if (r.ok && !nWarn && a === "update" && fin && fin.detail) sub = msg(fin.detail);
		$("#done-sub").textContent = sub;
		let html = "";
		if (!r.ok && r.error && !cancelled) html += `<div class="card"><div class="card-title">${esc(t("done_error"))}</div>${noteHtml(r.error, "error")}</div>`;
		if (nWarn) html += `<div class="card"><div class="card-title">${esc(t("done_warnings"))}</div>${r.warnings.map((w) => noteHtml(w, "warn")).join("")}</div>`;
		if (r.notes.length) html += `<div class="card"><div class="card-title">${esc(t("done_notes"))}</div>${r.notes.map((n) => noteHtml(n, "note")).join("")}</div>`;
		$("#done-body").innerHTML = html;
		$("#done-launch").hidden = !r.can_launch;
	}

	function logPath() {
		const r = S.result;
		const n = r && r.notes.find((x) => x.key === "uninstall_log" || x.key === "setup_log");
		if (n) return n.params.path;
		return `${info.install_dir}\\setup.log`;
	}

	function wire() {
		$("#btn-min").addEventListener("click", () => invoke("win_minimize"));
		$("#btn-close").addEventListener("click", async () => {
			if (!(await invoke("win_close"))) toast(t("close_blocked"));
		});
		$("#done-close").addEventListener("click", () => invoke("win_close"));
		$$(".lang-toggle button").forEach((b) => b.addEventListener("click", () => setLang(b.dataset.lang)));
		$$("[data-url]").forEach((b) => b.addEventListener("click", () => invoke("open_url", { url: b.dataset.url })));

		$("#actions").addEventListener("click", (e) => {
			const card = e.target.closest(".action");
			if (card && !card.disabled) selectAction(card.dataset.action);
		});
		$("#actions").addEventListener("dblclick", (e) => {
			const card = e.target.closest(".action");
			if (card && !card.disabled) { selectAction(card.dataset.action); showPage("readme"); }
		});
		$("#welcome-next").addEventListener("click", () => showPage("readme"));
		$$("[data-back]").forEach((b) => b.addEventListener("click", () => showPage(S.page === "options" ? "readme" : "welcome")));

		$("#readme").addEventListener("scroll", updateReadProgress, { passive: true });
		$("#readme").addEventListener("click", (e) => {
			const a = e.target.closest("a[href]");
			if (!a) return;
			e.preventDefault();
			const href = a.getAttribute("href");
			if (/^https:\/\//.test(href)) invoke("open_url", { url: href });
		});
		$("#read-check").addEventListener("change", (e) => {
			S.readChecked = e.target.checked;
			updateReadProgress();
		});
		$("#readme-next").addEventListener("click", enterOptions);

		$("#options-body").addEventListener("change", (e) => {
			const row = e.target.closest("[data-switch]");
			if (row) onSwitch(row.dataset.switch, e.target.checked);
		});
		$("#options-body").addEventListener("click", (e) => {
			if (e.target.closest("[data-retry]")) loadSource(S.action).then(() => loadPreflight(S.action));
			const zoom = e.target.closest("[data-zoom]");
			if (zoom) {
				openZoom(zoom.dataset.zoom);
				return;
			}
			const style = e.target.closest("[data-style]");
			if (style && S.opts && S.opts.visual_style !== style.dataset.style) {
				S.opts.visual_style = style.dataset.style;
				renderOptions();
			}
		});
		$("#lightbox").addEventListener("click", () => ($("#lightbox").hidden = true));
		$("#options-go").addEventListener("click", start);

		$("#toggle-log").addEventListener("click", () => {
			S.logOpen = !S.logOpen;
			const el = $("#log");
			el.hidden = !S.logOpen;
			if (S.logOpen) {
				el.textContent = logLines.join("\n");
				const sc = $("#progress-scroll");
				requestAnimationFrame(() => (sc.scrollTop = sc.scrollHeight));
			}
			$("#toggle-log span:last-child").textContent = t(S.logOpen ? "hide_details" : "show_details");
		});
		$("#cancel-btn").addEventListener("click", () => {
			invoke("cancel");
			$("#progress-note").textContent = t("cancelling");
		});

		$("#done-body").addEventListener("click", (e) => {
			const b = e.target.closest("[data-open]");
			if (b) invoke("open_path", { path: b.dataset.open });
		});
		$("#done-log").addEventListener("click", () => invoke("open_path", { path: logPath() }));
		$("#done-launch").addEventListener("click", async () => {
			try {
				await invoke("launch");
				toast(t("launched"));
				$("#done-launch").hidden = true;
			} catch (e) { toast(String(e)); }
		});

		document.addEventListener("contextmenu", (e) => { if (!e.target.closest(".readme, .log")) e.preventDefault(); });
		document.addEventListener("keydown", (e) => {
			const k = e.key.toLowerCase();
			if (k === "f5" || k === "f7" || (e.ctrlKey && ["r", "p", "f", "g", "j", "u", "s", "o"].includes(k)) || (e.ctrlKey && e.shiftKey && k === "i")) e.preventDefault();
			if (k === "end" && S.page === "readme" && document.activeElement !== $("#readme")) { $("#readme").scrollTop = 1e9; }
			if (k === "escape" && !$("#lightbox").hidden) $("#lightbox").hidden = true;
		});
		listen("setup-event", (ev) => onEvent(ev.payload));
		listen("close-blocked", () => toast(t("close_blocked")));
	}

	async function init() {
		info = await invoke("info");
		lang = info.lang === "pt" ? "pt" : "en";
		applyStatic();
		wire();
		renderWelcome();
		renderReadme();
		const page = info.page;
		if (page && actionAllowed(page)) {
			selectAction(page);
			showPage("welcome");
		} else if (!info.installed) {
			selectAction("install");
		}
		showPage("welcome");
		if (page && actionAllowed(page)) setTimeout(() => showPage("readme"), 450);
		requestAnimationFrame(() => invoke("ready"));
		checkLatest();
	}

	async function checkLatest() {
		let latest;
		try {
			latest = await invoke("check_latest");
		} catch (e) {
			return;
		}
		if (!latest || !latest.setup_newer) return;
		const note = $("#welcome-note");
		const next = $("#welcome-next");
		const wasDisabled = next.disabled;
		next.disabled = true;
		note.textContent = t("self_updating", { version: latest.version });
		try {
			await invoke("self_update");
		} catch (e) {
			note.textContent = t("self_update_failed", { error: String(e) });
			next.disabled = wasDisabled;
		}
	}

	init().catch((e) => {
		document.body.innerHTML = `<pre style="padding:20px;color:#ffb4ab;white-space:pre-wrap">${esc(e && e.stack ? e.stack : e)}</pre>`;
		invoke("ready");
	});
})();
