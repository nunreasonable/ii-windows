# Handoff: estado em 2026-10-02

Modo atual: **win11-gpu ligada** (o usuário cedeu a RTX em 2026-10-02 às 9h50). Ele usa o desktop da VM ao mesmo tempo: nada de injetar input, e nada de mudar o tema ou o papel de parede do Windows por conta própria.

Confirmado na VM em 2026-10-02:
- **ii:** sobe sem warnings novos, e o `formatKB` sumiu sem a tela fantasma (a hipótese se confirmou).
- **Notificações:** um toast do Windows é espelhado como popup do ii.
- **Atalhos:** o Settings não registra binds ("Not loading binds in settings.qml").
- **Settings:** aplica mudanças ao vivo (estilo da barra, cantos).
- **Wallpaper e cores:** o ii adota o wallpaper do Windows sem alterar o Windows, o matugen.exe gera o `colors.json` em `State/user/generated` e o relógio do fundo vai para a região menos ocupada.
- **Blur:** as janelas de backdrop ficam logo abaixo do painel, com o mesmo topmost (o render visual ainda não foi confirmado).
- **Desktops virtuais:** os painéis do ii não pertencem a desktop nenhum (`desktop=-1`). A barra sumia num desktop vazio porque o fundo do ii, em tela cheia, era tratado como "fullscreen app" (`ABN_FULLSCREENAPP`). Resolvido com `NonRudeHWND`; a barra continua no desktop 2.
- **Testado com o usuário ou com input injetado (o usuário liberou o uso da VM fora das horas em que ele olha):**
  - Tema claro/escuro e esquemas (com delay), desktops (Win+N), About, transparência/blur, overview com prévias, F11 e o cheatsheet no Win+/ (agora pelo layout ABNT2).
  - Seletor de wallpaper com miniaturas, ícones de pasta e caminhos DOS.
  - Recorte copiando a imagem (500x300 exato) e OCR do texto do cheatsheet.
  - Gravar sem ffmpeg avisa e não abre o seletor (bug de ordem corrigido).
  - Taskbar só no hover: aparece na borda, some 0,7 s depois de o cursor sair e não volta ao fechar apps. O teste precisa de `SendInput`, porque `SetCursorPos` não passa pelo hook LL.
- **Probes:** captura (monitor em ~43 ms, ao vivo, quadro único) e Bluetooth (sem adaptador) ok. O `net-probe` não roda por falta de uma DLL no pacote do próprio probe.
- **Crash:** um crash de teste depois de 12 s gera o dump e relança o shell (corrigido: era `-c` em vez de `-p`).
- **Revisão de código (2 agentes) aplicada:**
  - QML: o `TempScreenshotProcess.running` não voltava a false (também no Linux), a corrida do matugen, os retries quando o nativo carrega e o aviso do booru.
  - C++: relaunch, qFatal, error mode herdado, ordem do filtro, use-after-free no Thumbnailer, caminhos nativos no OCR e no wallpaper, ethernet exigindo gateway, retomada do barramento de notificações e o vazamento do clipboard.
  - Pendentes, de baixo impacto: corrida no `CaptureHandle` (live alternando), `captureScreen` sem limite de espera, toasts fantasmas ao religar o acesso e nome do perfil de Wi-Fi diferente do SSID.

## Branches

| Repo | Branch | Estado |
|---|---|---|
| `quickshell/` | `windows` | Integrado: bt, capture, notif, net, theme, region, pkg, blur. Também a varredura de janelas mortas, os workspaces "ocupados", os binds só no `shell.qml`, a camada Bottom abaixo das janelas, o servidor de notificação único por sessão e `ImageTools.imageSize`. |
| `ii/` | `windows` | Integrado: bt, notif, net, theme, region, blur, cmds. Também first-run/hyprlock sem bash, `/proc` desligado, a seção de lock escondida, o config do matugen gerado em tempo de execução (o do agente gravava fora do `State`) e o tamanho do wallpaper sem magick. |
| (integrado) | `theme` | matugen.exe 4.1.0 (cargo-xwin, em `toolchain/`), Wallpaper/ImageTools/Thumbnailer/FsUtils nativos e todos os chamadores de `switchwall.sh` redirecionados. Isso resolve o bug "configurações não mudam"; falta testar na VM (item 8). |
| (integrado) | `region` | Seletor de região no Windows: captura nativa, recorte (`Screenshot.cropToFile`), imagem no clipboard, OCR (`Ocr`, Windows.Media.Ocr num MTA), busca por `curl.exe`, edição no mspaint, gravação com `record.ps1` (ffmpeg gdigrab → mkv → mp4; som via loopback DirectShow). As regiões de conteúdo (OpenCV) ficam desligadas no Windows. Falta testar na VM. |
| (integrado) | `blur` | Blur atrás dos painéis por regra de camada (host backdrop do Windows.UI.Composition numa janela sem foco/clique sob cada painel). Só existe com a transparência do ii ligada. Plano B se o backdrop não renderizar: accent BLURBEHIND. Testes em `tools/testconfigs/blur` e `tools/blur-check.ps1`. Falta testar na VM. |
| (integrado) | `cmds` | Auditoria de todos os comandos Linux do ii: mapeados para APIs nativas (Hyprland.dispatch, Audio, ms-settings:, FsUtils/ImageTools, sendDesktop) ou desligados sem erro (warp-cli, cava, easyeffects, hyprpicker, trans, songrec, MicroTeX, ffplay). Revisado: `cmd /c "<comando com aspas>"` quebra porque o QProcess escapa aspas como `\"`, então o curl é chamado direto, o `>comando` do launcher vai pelo PowerShell e as listas do AI usam `FsUtils.listDir`. Regra: nunca passe a `cmd /c` um argumento com aspas; separe os argumentos ou use o PowerShell. |
| (integrado) | `pkg` | `tools/package.sh` → zip com `install.ps1`/`uninstall.ps1` (por usuário; autostart só com `-Autostart`). Crash handler Windows (minidump em `QsPaths::crashDir`, relança uma vez se o crash for depois de 10 s; teste com `QS_DEBUG_CRASH_TEST`/`QS_DEBUG_CRASH_DELAY_MS`). `build/qs` agora tem `CRASH_HANDLER=ON`. Nada disso rodou no Windows ainda (sem `pwsh` no host). |

No repo principal (sem worktrees), commite só caminhos explícitos: um `git add -A` leva arquivos de agentes trabalhando ao mesmo tempo.

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
8. **Tema (theme):** rodar `tools/deploy-ii.sh` (agora copia `toolchain/matugen.exe` para o lado do qs.exe, se existir) e `vm.sh push ii-windows` antes de `vm.sh ii start`. Depois, na sessão GUI:
   - Trocar o papel de parede pelo seletor (grid): confirma que a imagem muda de verdade no Windows (não só no `ii`) e que o grid mostra miniaturas reais, não só os ícones de pasta.
   - Alternar claro/escuro pela barra, pelo Quick Settings e por `qs ipc call theme toggleLightDark`: olhar o Settings do Windows (Personalização → Cores) para confirmar que mudou de verdade.
   - Trocar o tipo de paleta em Settings → Quick (Content, Expressive, etc.) e ver as cores do `ii` mudarem sem precisar trocar o papel de parede.
   - `qs ipc call wallpapers apply <caminho>`, launcher `accentcolor <hex>`, e os botões "Random"/"Choose file" (estes últimos sem a policy `weeb`, que ainda não tem equivalente Windows).
   - Fechar o `ii`, apagar `wallpaperPath` do config e reabrir: o fundo deve aparecer com o wallpaper que o Windows já tinha (não em branco).
   - Com `QT_LOGGING_RULES=quickshell.windows.wallpaper.debug=true;quickshell.windows.imagetools.debug=true;quickshell.windows.thumbnailer.debug=true`, confirmar que não há warnings de `matugen.exe not found` nem de imagem não decodificada.

## Bugs abertos

1. **`ResourcesPopup.qml`: "formatKB is not a function"** só aparece no Windows. No Linux, com o mesmo Qt, uma reprodução mínima não mostra o aviso. Hipótese mais forte: a tela fantasma `DISPLAY1` some no boot e a barra daquela tela é destruída, mas o conteúdo do popup (o `Row` dos valores) sobrevive e continua reavaliando com o `root` já morto. O endereço é sempre o mesmo, e as linhas 42/71 (valores constantes) nunca avisam. Se for isso, é um vazamento do upstream ao desplugar um monitor. Testar sem a tela fantasma; se confirmar, destruir o `contentItem` junto com o StyledPopup.
2. **Tela fantasma `\\.\DISPLAY1` 1280x800 no boot da win11-gpu.** É o vídeo emulado da VM e não é bug do código; gera warnings de `screen` nulo.
3. **Notificações entre processos (corrigido em c26f143, falta testar):** só o processo do `shell.qml` é o servidor (mutex `Local\\quickshell-notification-bus`). Os outros (settings, welcome) encaminham o `notifySend` por `WM_COPYDATA` para a janela `QuickshellNotificationBus` e não espelham os toasts.
4. **Warnings antigos:** a tradução `en_US.json`, o `commandPrefix` do AiChat/Anime (upstream) e a gamma do NightLight no VG259Q5A.

## Decisões recentes

- **O fundo do ii (camada Bottom) fica em HWND_BOTTOM, acima do desktop nativo.** O wallpaper do ii cobre os ícones, como no Linux, que não tem ícones. O Win+D revela o desktop nativo; o agente de tema define o mesmo wallpaper nele. Embutir o fundo no WorkerW, atrás dos ícones, fica como opção futura.
- **Global binds só carregam quando a entrada do processo é `shell.qml`.** Settings, welcome e killDialog não registram atalhos.

## Próximas fases

- **Fase 4:** blur atrás dos painéis, conferir OSD, OSK e overview com prévias, e WorkerW se o usuário quiser ícones visíveis.
- **Fase 6:** integrar `pkg` e decidir o autostart com o usuário.

## Terminal (2026-10-02): feito e testado na VM

- **Cores:** `Quickshell.Windows.TerminalColors` em C++, bit a bit igual ao `generate_colors_material.py`, conferido no host (1,6M cores) e no Windows (5312 casos). `services/WindowsTerminalTheme.qml` gera:
  - `State/user/generated/terminal/sequences.txt`, como no Linux mas sem o OSC 1, que no Windows Terminal vira título de aba;
  - o fragmento `%LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\illogical-impulse\illogical-impulse.json`, com o esquema e as atualizações dos perfis Windows PowerShell, cmd e PowerShell 7 (fonte, cursor em barra, padding 22, acrílico conforme a transparência do ii);
  - `ii.omp.json`, o tema do Oh My Posh no layout do `starship.toml` do ii.
  Ao mudar, ele toca a data do `settings.json` para o Windows Terminal recarregar. Com `enableTerminal` desligado, apaga os três.
- **Shell:** `defaults/windows/terminal/profile.ps1` usa o Oh My Posh com o tema do ii e, se ele não existir, o Starship com o `starship.toml` do ii. Ele também imprime o `sequences.txt` e define os aliases clear/celar/claer, `ls` → eza e `q`. O `starship.toml` agora é uma cópia byte a byte: a do agente perdia os glifos de uso privado.
- **O que o instalador precisa fazer, tudo descrito no README:**
  - Instalar a fonte JetBrainsMono NF por usuário: copiar os TTFs de `dist/fonts` para `%LOCALAPPDATA%\Microsoft\Windows\Fonts` e registrar em HKCU `...\Fonts`, com WM_FONTCHANGE. Sem admin.
  - Instalar por `winget --scope user`, sem UAC: `JanDeDobbeleer.OhMyPosh`, `Starship.Starship`, `eza-community.eza`. O `Microsoft.PowerShell` (PowerShell 7) exige UAC e fica opcional.
  - No `$PROFILE` do PowerShell 7 e no do 5.1, adicionar o bloco `# >>> illogical-impulse >>>` / `# <<< illogical-impulse <<<`, que dot-sourcea o `profile.ps1` com Test-Path.
  - O PowerShell 5.1 só carrega o perfil com a ExecutionPolicy CurrentUser em RemoteSigned. É uma mudança de segurança: precisa de opção explícita no instalador e de menção no README.
  - Guardar num manifesto o que foi instalado, para o desinstalar remover só o que o ii pôs.
- **Já instalados na VM durante os testes:** a fonte por usuário, Starship, eza e Oh My Posh, todos no escopo do usuário.

## Instalador GUI (2026-10-02): feito e testado na VM

- `installer/` (Tauri v2, Rust + HTML/CSS/JS estático) gera o `ii-windows-setup.exe` (~7,5 MB). Ações: instalar, atualizar, reparar e desinstalar, tudo por usuário, com o README (pt-BR/en) obrigatório antes. Detalhes em `docs/PORTING.md`, seção "GUI installer". `tools/release.sh` monta `dist/release/` a partir do `VERSION`.
- **Testado na VM (offline, sem release no GitHub), na ordem:** instalar → reinstalar → atualizar 0.1.0→0.1.1 pela cópia instalada (relança do `%TEMP%`) → atualizar a mesma versão (forçado) → GitHub sem release (404 tratado) → reparar → desinstalar (`UninstallString`) com restauração de papel de parede, claro/escuro, cor de destaque e auto-hide → instalar de novo (fontes e eza instalados pelo setup) → desinstalar (eza e fontes removidos, configurações mantidas) → cancelar no meio (rollback limpo) → download por HTTP local com checksum errado (rejeitado) e certo → reinstalar. Cada item do manifesto foi conferido depois do uninstall.
- **Não testado:** a opção do PowerShell 7 (precisa do UAC, que não aceita input injetado) e a opção da ExecutionPolicy (a CurrentUser já estava `RemoteSigned`, então ela aparece como "já permitido"); download real do GitHub (não existe release).
- **Bug achado no qs:** `qs kill --pid <ii>` faz o ii sair do loop ("Exiting due to IPC request", "Blur off ...") e travar com uma thread só, sem restaurar a barra de tarefas (`QEventDispatcherWin32::wakeUp: Failed to post a message (Invalid window handle.)`). Acontece sempre. O instalador espera 10 s, encerra o processo, mostra a barra e volta o auto-hide anotado.
- **Notado no ii:** depois de um reparo (config padrão), o ii adota `TranscodedWallpaper`, que nesta VM tem uma imagem antiga (bloom); o papel de parede real é por monitor (`IDesktopWallpaper::GetWallpaper(null)` não é o que aparece na tela). O ii não mudou o papel de parede do Windows.
- **Estado final da VM:** ii instalado pelo setup e rodando de `%LOCALAPPDATA%\ii-windows`, com as configurações do usuário restauradas do backup `C:\ii-windows\backup-20261002-124011` (que ficou lá). As fontes JetBrainsMono e o eza agora pertencem ao manifesto do setup; Oh My Posh e Starship continuam do usuário. A cópia de dev `C:\ii-windows\ii-windows` ficou, mas não está rodando.

