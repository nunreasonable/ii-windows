#!/usr/bin/env python3
import argparse
import pathlib
import re
import sys

ASSIGN = re.compile(r"Config\.options\.([A-Za-z_][A-Za-z0-9_.]*?)\s*(?:=(?!=)|\+=|-=|\.(?:push|splice|pop|shift|unshift)\()")
NESTED = re.compile(r"Config\.setNestedValue\(\s*[\"'`]([^\"'`]+)[\"'`]")
PAGES = {
    "quick": "QuickConfig.qml", "general": "GeneralConfig.qml", "bar": "BarConfig.qml",
    "desktop": "BackgroundConfig.qml", "interface": "InterfaceConfig.qml", "services": "ServicesConfig.qml",
    "system": "SystemConfig.qml", "profile": "Profile.qml", "about": "About.qml",
}
CARD_FILES = {
    "style": "DashboardStyleCard.qml", "visualstyle": "DashboardVisualStyleCard.qml", "schemes": "DashboardSchemeCard.qml",
    "barpos": "DashboardBarPositionCard.qml", "barlayout": "DashboardBarLayoutCard.qml",
    "collagelayouts": "DashboardCollageLayoutsCard.qml", "widgets": "DashboardWidgetsCard.qml", "weathermap": "DashboardWeatherMapCard.qml",
    "webapps": "DashboardWebappsCard.qml", "iconpicker": "DashboardIconCard.qml", "timepreview": "DashboardTimeCard.qml",
}
CARD_KEYS = {"barlayout": ["bar.layouts.leftLayout", "bar.layouts.middleLayout", "bar.layouts.rightLayout"]}
COMMENT = re.compile(r"/\*.*?\*/|^[ \t]*//.*$", re.S | re.M)


def writes(text):
    text = COMMENT.sub("", text)
    return {m.group(1) for p in (ASSIGN, NESTED) for m in p.finditer(text)}


def block_end(text, start):
    depth = 0
    i = start
    quote = None
    while i < len(text):
        c = text[i]
        if quote:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                quote = None
        elif c in "\"'`":
            quote = c
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return len(text)


def own_title(body):
    depth = 0
    for line in body.split("\n")[1:]:
        if depth == 0:
            m = re.match(r'\s*title:\s*Translation\.tr\("((?:[^"\\]|\\.)*)"\)', line)
            if m:
                return m.group(1)
        depth += line.count("{") - line.count("}")
    return None


def find_block(text, kinds, title):
    for m in re.finditer(r"\b(?:" + "|".join(kinds) + r")\s*\{", text):
        start = text.index("{", m.start())
        end = block_end(text, start)
        if own_title(text[start:end]) == title:
            return text[start:end]
    return None


def main():
    parser = argparse.ArgumentParser(description="Check that every Config key the classic end4-pC pages write is reachable from the dashboard")
    parser.add_argument("root", type=pathlib.Path, help="the ii checkout (e.g. ii-windows/ii)")
    args = parser.parse_args()
    pages_dir = args.root / "modules/settingsPc/pages"
    dash = args.root / "modules/settingsPc/dashboard"
    catalog = (dash / "DashboardSettingsCatalog.qml").read_text(encoding="utf-8")
    page_texts = {pid: (pages_dir / f).read_text(encoding="utf-8") for pid, f in PAGES.items()}

    wanted = {}
    for pid, text in page_texts.items():
        for key in writes(text):
            wanted.setdefault(key, []).append(pid)

    reached = {}

    def add(key, why):
        reached.setdefault(key, why)

    for m in re.finditer(r'key:\s*"([^"]+)"', catalog):
        if ":" not in m.group(1):
            add(m.group(1), "card")
    for m in re.finditer(r'"(custom:[A-Za-z0-9_]+)"\s*:\s*\{', catalog):
        start = catalog.index("{", m.end() - 1)
        for key in writes(catalog[start:block_end(catalog, start)]):
            add(key, m.group(1))
    used_types = set(re.findall(r'type:\s*"([a-z]+)"', catalog))
    for t, f in CARD_FILES.items():
        if t in used_types and (dash / f).exists():
            for key in writes((dash / f).read_text(encoding="utf-8")) | set(CARD_KEYS.get(t, [])):
                add(key, f)
    missing_sections = []
    for m in re.finditer(r'\{[^{}]*type:\s*"page"[^{}]*\}', catalog):
        obj = m.group(0)
        pid = re.search(r'pageId:\s*"([^"]+)"', obj)
        sec = re.search(r'section:\s*"([^"]+)"', obj)
        sub = re.search(r'subsection:\s*"([^"]+)"', obj)
        if not pid or pid.group(1) not in page_texts:
            missing_sections.append(obj)
            continue
        text = page_texts[pid.group(1)]
        scope = text
        if sec:
            scope = find_block(text, ["ContentSection"], sec.group(1))
            if scope is None:
                missing_sections.append(obj)
                continue
        if sub:
            narrowed = find_block(scope, ["ContentSubsection"], sub.group(1))
            if narrowed is None:
                missing_sections.append(obj)
                continue
            scope = narrowed
        for key in writes(scope):
            add(key, "page:" + pid.group(1) + "/" + (sec.group(1) if sec else "") + ("/" + sub.group(1) if sub else ""))

    for obj in missing_sections:
        print("page card points at nothing:", " ".join(obj.split()))
    unreached = sorted(k for k in wanted if k not in reached)
    for key in unreached:
        print(f"not reachable from the dashboard: {key}  (written by {', '.join(wanted[key])})")
    print(f"{len(wanted) - len(unreached)} of {len(wanted)} keys written by the classic pages are reachable, {len(missing_sections)} broken page cards")
    return 0 if not unreached and not missing_sections else 1


if __name__ == "__main__":
    sys.exit(main())
