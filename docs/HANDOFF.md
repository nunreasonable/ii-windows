# Handoff: estado em 2026-10-01 (sessão congelada)

Tudo está commitado localmente. Nenhum agente está rodando. Para retomar, leia isto, `docs/PORTING.md` e `docs/AGENTS.md`.

## Branches

| Repo | Branch | Estado |
|---|---|---|
| `quickshell/` | `windows` | Integrado e testado na VM, incluindo a varredura de janelas mortas e os workspaces "ocupados" ao estilo Hyprland (último commit). |
| `ii/` | `windows` | Integrado e testado na VM. |
| `quickshell` worktree `wt-net` | `net` | Backend WlanAPI/NLM commitado pelo agente. Falta revisar e integrar. |
| `ii` worktree `wt-ii-net` | `net` | WIP: `Network.qml`, WifiDialog, WifiControl. O agente parou no meio do WifiControl (waffle). |
| `quickshell` `wt-bt` | `bt` | Núcleo WinRT commitado. Faltam as classes QML (adapter, device, singleton). |
| `ii` `wt-ii-bt` | `bt` | Nenhuma mudança ainda. |
| `quickshell` `wt-theme` | `theme` | WIP: `image/image_tools` (least_busy_region, text_color, scheme_for_image), thumbnailer, `system/wallpaper`. O agente estava redeployando para testar. |
| `ii` `wt-ii-theme` | `theme` | WIP: template e config do matugen em `defaults/windows/matugen`, ganchos no WindowsNative. Falta o `matugen.exe` (cargo-xwin) e o Wallpapers.qml. |
| `quickshell` `wt-capture` | `capture` | ScreencopyView por WGC commitado. O agente estava rodando o probe na VM. Falta validar e integrar. |
| `quickshell` `wt-notif` / `ii` `wt-ii-notif` | `notif` | Servidor de notificações commitado nos dois lados. O agente estava escrevendo a config de teste sem janela (`tools/testconfigs/notifications`). |

Integração: para cada um, faça o merge na branch `windows`. Os conflitos de sempre ficam em `src/windows/CMakeLists.txt` e `src/windows/services/CMakeLists.txt`: junte as listas de fontes e os `add_subdirectory`. Depois, build com `cmake --build build/qs`, `tools/deploy-ii.sh`, `tools/vm.sh push ii-windows` e `tools/vm.sh ii start`. Por fim, remova a worktree.

## Bugs abertos, achados no último teste do usuário

1. **As configurações do app de Settings não chegam ao `config.json`.** Na VM, `%LOCALAPPDATA%\illogical-impulse\config.json` tinha sido gravado pela última vez às 18:23, mas o usuário mexeu nas configurações por volta das 21h. O log não mostra nenhum "Write of ... failed". Próximo passo: rodar `qsw -p settings.qml` com `QT_LOGGING_RULES=quickshell.io.fileview.debug=true` e ver se `writeAdapter` é chamado e para qual caminho (suspeita: `Directories.config` ou `shellConfigPath` com `/C:/`, ou `adapterUpdated` não disparando no processo do settings).
2. **O processo do Settings também registra os atalhos globais.** Ele carrega o `keybinds.json` e instala o hook de teclado, então os atalhos ficam duplicados enquanto ele está aberto. Os binds devem carregar só no shell principal (por exemplo, de forma preguiçosa, no primeiro GlobalShortcut, ou nunca com `-p settings.qml`).
3. **`ResourcesPopup.qml`: "formatKB is not a function".** O aviso se repete a cada atualização. A função está na raiz, que é um LazyLoader (StyledPopup). Conferir se acontece no Linux. A correção provável é mover a função para dentro do conteúdo ou usar uma função livre.
4. **Na partida, uma tela `\\.\DISPLAY1` de 1280x800 aparece por um instante.** Ela gera warnings de `screen` nulo em Background, ScreenCorners, Lock e Brightness. O display fantasma foi desconectado, mas o Qt ainda o vê no boot.
5. **`least-busy-region-venv.sh`** ainda é chamado no Windows. O C++ equivalente está na branch `theme`.
6. **Warnings antigos que continuam:** FileViews de `/proc/*` e `/etc/os-release`, a tradução `en_US.json`, o `commandPrefix` do AiChat e a gamma do NightLight (falha no VG259Q5A).

## Corrigido nesta sessão (já na branch `windows`)

- **Janelas fechadas continuavam na barra.** Apps que encerram com ExitProcess (NVIDIA App, kill, crash) não mandam `EVENT_OBJECT_DESTROY`. O tracker agora confere `IsWindow` a cada troca de foreground e a cada 2 s. Testado: notepad morto por `Stop-Process` sai da lista.
- **Todos os desktops apareciam como ocupados.** `Hyprland.workspaces` agora lista só desktops com janelas mais o focado, como no Hyprland, com eventos `createworkspace`/`destroyworkspace`. O `workspace empty` do dispatcher procura em todos os desktops.
- **Cliques na barra:** o FocusGrab tem 500 ms de tolerância. Nomes amigáveis dos apps no ActiveWindow.

## Próximas fases

Depois de integrar a leva 2 e corrigir os bugs acima:

- **Fase 4/5:** WorkerW para a camada de fundo, blur, região/OCR com Windows.Media.Ocr, gravação com ffmpeg ddagrab, conferir OSD e OSK.
- **Fase 6:** pacote, autostart (perguntar antes) e crash handler.
