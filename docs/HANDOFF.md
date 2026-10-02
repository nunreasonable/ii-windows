# Handoff: estado em 2026-10-02

Modo atual: **só build**. O usuário precisa da RTX no Linux, então as duas VMs ficam desligadas e não se liga nenhuma. Tudo o que foi integrado desde 2026-10-01 compila, mas ainda não rodou na VM (ver "Testes pendentes").

## Branches

| Repo | Branch | Estado |
|---|---|---|
| `quickshell/` | `windows` | Integrado: bt, capture, notif, net. Também a varredura de janelas mortas, os workspaces "ocupados", os binds só no `shell.qml` e a camada Bottom abaixo das janelas. |
| `ii/` | `windows` | Integrado: bt, notif, net. Também as correções de first-run/hyprlock, os `/proc` e a seção de lock escondida no Windows. |
| `quickshell` `wt-theme` / `ii` `wt-ii-theme` | `theme` | Agente trabalhando. Escopo: matugen, wallpaper, modo escuro e **todos** os chamadores de `switchwall.sh`. É o bug "configurações não mudam" do usuário: Settings → Quick chama bash. |
| `wt-region` / `wt-ii-region` | `region` | Agente trabalhando: seletor de região, OCR (Windows.Media.Ocr), recorte/cópia, busca de imagem, gravação com ffmpeg. |
| `wt-pkg` | `pkg` | Agente trabalhando: `tools/package.sh`, `install.ps1`/`uninstall.ps1` (autostart só com `-Autostart`), crash handler com minidump. |

Integração: faça o merge na `windows`. Os conflitos típicos ficam em `src/windows/CMakeLists.txt` (juntar as listas de fontes e de libs) e em `ii/modules/common/WindowsNative*.qml` (juntar as propriedades). Depois rode `. tools/env.sh && cmake build/qs && cmake --build build/qs` e remova a worktree.

## Testes pendentes (quando a VM voltar)

Cada agente deixou os passos no próprio relatório. Os resumos estão em `docs/PORTING.md`, e os probes em `tools/*-probe` e `build/*-probe`. Antes de testar, rode `tools/deploy-ii.sh`, que agora limpa os shims velhos de `dist/`.

1. **Tracker e workspaces:** abrir e fechar apps, matar processos; os pontos de workspace só aparecem nos ocupados.
2. **Atalhos com o Settings aberto:** cada atalho dispara uma vez só.
3. **Camada Bottom (fundo do ii):** fica abaixo das janelas e não cobre apps. Win+D mostra o desktop nativo, o que é esperado.
4. **Bluetooth** (VM sem adaptador): a UI some e não há TypeErrors.
5. **Captura:** `capture-probe`, depois as prévias do overview e do dock.
6. **Notificações:** `tools/notif-test.ps1`, depois um toast da Calculadora espelhado no ii.
7. **Rede:** `net-probe`, depois o ícone de ethernet e o diálogo de Wi-Fi sem adaptador.

## Bugs abertos

1. **`ResourcesPopup.qml`: "formatKB is not a function"** só aparece no Windows. No Linux, com o mesmo Qt 6.11.2, uma reprodução mínima não mostra o aviso. Investigar com a VM (cache de QML? lookup do método numa raiz LazyLoader?).
2. **Tela fantasma `\\.\DISPLAY1` 1280x800 no boot da win11-gpu.** É o vídeo emulado da VM e não é bug do código; gera warnings de `screen` nulo.
3. **`sendDesktop` de processos separados** (welcome.qml) vai para o servidor de notificação do próprio processo. Precisa de uma rota por IPC.
4. **Warnings antigos:** a tradução `en_US.json`, o `commandPrefix` do AiChat/Anime (upstream) e a gamma do NightLight no VG259Q5A.

## Decisões recentes

- **O fundo do ii (camada Bottom) fica em HWND_BOTTOM, acima do desktop nativo.** O wallpaper do ii cobre os ícones, como no Linux, que não tem ícones. O Win+D revela o desktop nativo; o agente de tema define o mesmo wallpaper nele. Embutir o fundo no WorkerW, atrás dos ícones, fica como opção futura.
- **Global binds só carregam quando a entrada do processo é `shell.qml`.** Settings, welcome e killDialog não registram atalhos.

## Próximas fases

- **Fase 4:** blur atrás dos painéis, conferir OSD, OSK e overview com prévias, e WorkerW se o usuário quiser ícones visíveis.
- **Fase 6:** integrar `pkg` e decidir o autostart com o usuário.
