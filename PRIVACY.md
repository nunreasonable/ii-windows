# Privacy

ii-windows has no telemetry and no analytics, and it sends nothing to its developers. Your settings, notes, to-dos, AI chats and logs stay on your PC, in `%LOCALAPPDATA%\illogical-impulse` and `%LOCALAPPDATA%\quickshell`.

Some features talk to other services over the internet. This page lists all of them, what they send and when. Each of those services has its own privacy policy.

## Automatic

These happen without you opening a feature.

- **Update check (ii).** Once at startup and then once a day, ii asks GitHub (`api.github.com`) for the latest ii-windows release. The request carries no personal data beyond what any web request does (your IP address). Both the check and its notification can be turned off in Settings → About, with the "Check for updates automatically" and "Notify me when a new version is available" switches.
- **Update check (setup).** Every time you open `ii-windows-setup.exe`, it asks GitHub for the latest release. If its own copy is older, it downloads the newer setup from the GitHub release, checks it against the SHA-256 that GitHub publishes, and reopens with it. It doesn't do this when uninstalling.
- **App updates count.** At startup and every 2 hours, ii runs `winget upgrade` to count apps with updates. winget contacts its configured sources (Microsoft's by default). Turn it off in Settings → Services → App updates, with the "Count apps with available updates" switch.
- **Weather (off by default).** If you turn the weather widget on, ii asks `wttr.in` for the forecast of the city you typed, every 10 minutes by default.

## Only when you use a feature

- **AI chat:** your messages, the system prompt and any file you attach go to the provider you pick:
  - Google Gemini (`generativelanguage.googleapis.com`);
  - OpenRouter (`openrouter.ai`);
  - Mistral (`api.mistral.ai`);
  - a local Ollama or vLLM server, which stays on your PC.

  The system prompt includes the date, your desktop and the active window's app name. API keys are kept in Windows Credential Manager and sent only to their provider.
- **AI usage tab** (Cheatsheet): while this tab is open, ii reads the sign-in token that Claude Code or Codex already saved on your PC and sends it only to that tool's company (`api.anthropic.com` or `chatgpt.com`) to show your plan usage. It does this every 60 seconds while the tab is open. Without those tools nothing is sent.
- **Translator** (sidebar, off by default) and **screen translator:** the text to translate goes to Google Translate (`translate.googleapis.com`). The text recognition for the screen translator runs on your PC.
- **Music recognition:** while it's listening, a short audio fingerprint of what is playing goes to Shazam through the bundled SongRec.
- **Lyrics:** the title, artist and length of the song that is playing go to `lrclib.net` while the lyrics are on screen.
- **Image search** from the screenshot tool: the region you selected is uploaded to `uguu.se` (an anonymous file host), and your browser opens Google Lens with that link.
- **Web search:** your browser opens your search engine (Google by default) with what you typed.
- **Wallpapers and images:** your search terms go to the service you browse:
  - Wallhaven;
  - Unsplash or Pexels, with your API key;
  - Konachan or osu! (random wallpaper buttons);
  - the curated lists on GitHub;
  - the booru sidebar (yande.re, Konachan, Zerochan, Danbooru, Gelbooru, waifu.im, alcy.cc). For Zerochan, your configured username is sent if you set one.
- **Favicons:** when a search result or an AI answer shows a link, its domain is sent to Google's favicon service (`www.google.com/s2/favicons`) to get the icon.
- **Community presets:** Settings → Profile reads the preset list from GitHub. Publishing a preset uses your GitHub sign-in and creates a pull request with the files you chose.
- **Web apps:** when you add a website as an app, ii loads that site and its icon.
- **Auto EQ** (off by default, needs your own Last.fm key): the artist and title of the song playing in Spotify or YouTube go to Last.fm on each track change, to pick a genre preset.
- **Updating from the notification:** the new setup is downloaded from the GitHub release.
- **Setup options:**
  - the setup downloads the release from GitHub;
  - it can install the terminal tools, PowerShell 7 and FFmpeg with winget;
  - on Windows 10 without winget, it can download winget itself from Microsoft's winget-cli release on GitHub.

## Report a problem

Settings → About → "Report a problem" only creates a zip on your Desktop and opens a new GitHub issue in your browser. Nothing is uploaded: you decide whether to attach the zip. Your Windows user name is removed from the paths in it.

## Questions

Open an issue at https://github.com/nunreasonable/ii-windows/issues.
