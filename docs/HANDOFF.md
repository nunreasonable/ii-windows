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
- **Bug achado no qs (corrigido, `a7eace3`):** `qs kill --pid <ii>` travava o ii com uma thread só: a `VirtualDesktopAccessor.dll` faz chamadas COM no DLL detach do `ExitProcess`. O `main` agora termina com `TerminateProcess` depois da limpeza do Qt; o ii sai em menos de 1 s. O instalador continua com o fallback (espera 10 s, encerra, mostra a barra, volta o auto-hide). Um `qs kill -c ii` ainda trava se houver um processo zumbi antigo com a mesma config (ele espera a resposta do zumbi); use `--pid`.
- **Notado no ii (corrigido, `15a965f`):** o ii adotava `TranscodedWallpaper` em vez do papel de parede real, que é por monitor. `currentWallpaper()` agora pergunta a cada monitor antes de cair no `SPI_GETDESKWALLPAPER`.
- **Revisão do README (2026-10-02):** o texto dizia menos do que o ii faz. Agora diz que a cor de destaque do Windows segue a paleta do ii toda vez que ela carrega (inclusive ao abrir), que a luz noturna do ii é rampa de gama (volta ao fechar), que o brilho de monitor externo vai por DDC/CI e fica, e cita o histórico da área de transferência e a credencial `illogical-impulse` (chaves de API do painel de IA).
- **Achados da revisão, corrigidos:** as imagens do histórico da área de transferência (`%LOCALAPPDATA%\cache\quickshell\clipboard`, o `GenericCacheLocation` do Qt) nunca eram apagadas entre execuções. Agora cada `Clipboard` usa `<pid>-<n>`, apaga a sua pasta ao ser destruído e, na primeira criação do processo, apaga pastas de processos mortos. O handler de `WM_CLIPBOARDUPDATE` usa `QPointer` (o singleton é recriado num reload). A desinstalação agora apaga `%LOCALAPPDATA%\cache\quickshell`, apaga `cache\thumbnails` (miniaturas do ii) só se a pasta não existia antes (anotado no `pre_install`; manifestos antigos não têm o campo e a pasta fica) e remove a credencial `illogical-impulse` quando as configurações não são mantidas. Testado na VM: atualização forçada pela GUI com o pacote novo, cache criado e apagado ao sair, sobras antigas apagadas ao abrir, `CredDeleteW` com e sem credencial.
- **Estado final da VM:** ii instalado pelo setup e rodando de `%LOCALAPPDATA%\ii-windows`, com as configurações do usuário restauradas do backup `C:\ii-windows\backup-20261002-124011` (que ficou lá). As fontes JetBrainsMono e o eza agora pertencem ao manifesto do setup; Oh My Posh e Starship continuam do usuário. A cópia de dev `C:\ii-windows\ii-windows` está presa pelo `qsw` zumbi PID 6544 (uma thread, não morre; precisa sair da sessão ou reiniciar a VM). O release atual está em `C:\ii-windows\release`.


## Publicação, travamento e Windows 10 (2026-10-02, tarde)

- **Publicado:** repo `nunreasonable/ii-windows` (público, GPL-3.0, README com créditos), releases v0.1.0 e v0.1.1; forks `nunreasonable/quickshell` (`windows`) e `nunreasonable/dots-hyprland` (`ii-windows`) atualizados junto. O histórico do repo principal foi reescrito antes de publicar para tirar usuário/IP do host (backup em bundle no scratchpad da sessão).
- **Travamento de mouse/teclado no PC real (i3-13100, iGPU, Win11 26H2):** v0.1.1 tirou os WinEvent hooks da thread da GUI (thread própria que descarta eventos de cursor/caret e entrega os de janela em lote), deixou o hook do mouse em prioridade alta, desligou a coleta de GPU (o ii não mostra) e loga entrada atrasada nos hooks (`Input reached the hooks late`, em `qs log`). Não confirmado no PC real ainda.
- **Windows 10 (VM `win10`, 22H2 19045, criada por `tools/vm-win10.sh`, instalação autônoma com `tools/win10/autounattend.xml`; credenciais em `build/win10-vm/credentials`; chave do agente `iiw-agent-win10`):**
  - Runtime do VC++ não vinha no pacote: `qs.exe` não abria em PC sem o redistribuível (0xC0000135). Agora `tools/deploy.sh` copia as 6 DLLs de `toolchain/vcredist` (do VC_redist do VS 2026, 14.51, ≥ headers 14.44).
  - `win10\VirtualDesktopAccessor.dll` (release `2019-windows10` do Ciantic), escolhido por `windowsBuild()`; criar área de trabalho = Ctrl+Win+D, remover não existe. Testado: troca e criação com Win+N.
  - Blur: caminho `SetWindowCompositionAttribute` (blur behind), uma janela por formato, porque esse blur ignora `SetWindowRgn` (faixa desfocada sob a barra; corrigido e testado).
  - Barra de tarefas hover-only em qualquer borda (testado só embaixo); popups do Win10 (SearchApp etc.).
  - `exec` com alternativas `a || b`; programas de console abrem com console próprio e sem os std handles do shell (antes o PowerShell abria escondido ou fechava na hora). Testado: Win+Enter abre o PowerShell.
  - Logos: `windows10-symbolic`/`microsoft-symbolic` (barra) e `windows10-logo`/`windows11-logo` coloridos (aba Sobre, que não recolore).
  - Instalador: checagem do WebView2, Windows 10 2004+ liberado (abaixo de 19041 bloqueia), Windows Terminal via winget quando falta, winget (App Installer) baixado do release oficial (msixbundle + dependências x64, ~300 MB, SHA-256 do `.txt` do release) no Win10 sem winget, removido na desinstalação só se o setup instalou. O download do winget nunca rodou de verdade.
  - Brilho: no Windows o ii sempre assumia DDC/CI (notebooks nunca funcionavam). Agora `Brightness.probe()` pergunta só ao WMI ao abrir; DDC/CI quando o usuário muda; se o DDC/CI falha, `GammaController` escurece pela gama (o Windows aceita até ~50%; o controle é mapeado nessa faixa). A luz noturna usa o mesmo controlador. Testado na VM (sem DDC): 90/80/70/60/50% e volta à rampa original.
- **Publicado como v0.2.0** (commits nos três repos e release). O usuário testou o primeiro build 0.2.0 na VM (antes do winget, do brilho e do blur por formato).

## 0.3.0 (2026-10-05): workflow multiagente

- **Como foi feito:** um workflow com 11 frentes (Opus nas difíceis, Sonnet no resto), cada uma num worktree próprio, seguida de um revisor (Sonnet) que corrigiu o que achou. Integração e testes na VM `win11` sem GPU (WARP), por mim.
- **Upstream:** Quickshell 0.3.1+ (95 commits) mesclado no `windows`; ii do end-4 (main de 2026-10-03) mesclado no `windows` do ii.
- **Gravação nativa** (`ScreenRecorder`, WGC + Media Foundation H.264, áudio por loopback WASAPI): testada, 600x400 a 30 fps, sem quadros perdidos, mp4 válido. O ffmpeg fica só como alternativa; o instalador oferece o FFmpeg via winget (desligado por padrão).
- **Bandeja do sistema** (`services/systray`): janela `Shell_TrayWnd` escondida na frente da do Explorer, repassando tudo a ele; ícones já existentes vêm do `ITrayNotify`. Testado: ícones na barra, botão direito abre o menu do app. **Bug achado e corrigido:** sem as propriedades da janela do Explorer (`TaskbandHWND` etc.), a `ITaskbarList3` devolvia E_NOTIMPL e apps WPF com progresso na barra (o WinUtil do Chris Titus) caíam no `ShowDialog`. A janela agora espelha as propriedades do Explorer a cada segundo.
- **Fundo nativo** (`desktop_host.cpp`, `DesktopLayer`): o fundo do ii vira filho da `WorkerW` (24H2+: dentro do Progman, abaixo da `SHELLDLL_DefView`). Testado no 25H2. Na VM `win11` os ícones estão escondidos por configuração do usuário (`HideIcons = 1`), não pelo ii.
- **Tradução:** aba do tradutor e tradutor de tela sem `trans`/GCloud. O Google responde ao `curl.exe` do Windows com a página "unusual traffic" (até com agente de navegador); agora é `XMLHttpRequest` do QML. Testado: português para inglês.
- **Música:** `songrec.exe` (SongRec 0.7.5 + interface Windows em `tools/songrec`, GPL), loopback WASAPI pelo cpal. Testado só o caminho sem música (sai 0, sem resultado).
- **LaTeX:** `LaTeX.exe` (MicroTeX com CLI Qt/SVG própria, `tools/microtex`). Testado: SVG certo de `\frac{a}{b} + \sqrt{x^2}`.
- **Terminal no conhost:** banner apagado e cores do ii; os glifos Nerd Font aparecem como quadrados porque o conhost usa a fonte dele.
- **Robustez:** corrida do `CaptureHandle`, espera sem limite do `captureScreen`, toasts fantasmas, perfil de Wi-Fi com nome diferente do SSID.
- **Instalador:** ao abrir consulta o último release; se for mais novo que ele mesmo, baixa o `ii-windows-setup.exe` do release, confere com o `digest` SHA-256 do GitHub e reabre (`--relaunched --session --no-self-update --after <pid>`). Pacote local mais velho perde para o release. Testado com um release falso servido de dentro da VM.
- **Merge com problema achado só em execução:** duas frentes criaram o mesmo sinal `desktopActionInvoked` em `Notifications.qml`; o `qmlformat` não pega isso, e o ii não abria. Para merges grandes de QML, testar rodando.

## 0.4.0 (2026-10-05): limitações conhecidas

- **Como foi feito:** workflow com 4 frentes (tiling, bandeja, widgets do desktop, prompt do conhost) e revisores; integração, testes e correções por mim na VM `win10` (sem GPU). Depois, um agente para o tradutor de tela.
- **Tiling nativo** (`tiling.{hpp,cpp}`, `tiling_layout.{hpp,cpp}`, singleton `Tiling`): dwindle do Hyprland por (desktop virtual, monitor), sem GlazeWM. Desligado por padrão (`windowsPort.tiling` no config, seção "Windows tiling" nas Configurações). Testado: layout abaixo da barra, desligar devolve as janelas ao lugar. **Bug corrigido:** ao ligar com janelas já abertas, todas as divisões saíam lado a lado (boxes vazios na inserção).
- **Bandeja no Win10:** ícones de apps abertos antes do ii ganham a mensagem de callback pelas toolbars do Explorer ou por `TaskbarCreated` direcionado. A 1024 px a barra do ii esconde a bandeja (upstream), então o clique foi visto só no log. **Bug corrigido:** o gancho reagia a eventos de janela do Explorer (WinEvent de show/foreground) subindo para `HWND_TOPMOST` no meio do processamento da AppBar, e a barra perdia a reserva de espaço; o gancho voltou a se reposicionar só pelo timer de 100 ms. `TaskbarCreated` não vai mais para janelas do Explorer.
- **Widgets do desktop:** janela própria filha do dono da `SHELLDLL_DefView`, acima dela; a máscara vira a região da janela. Testado: relógio arrastável (com posicionamento "free") e ícones clicáveis.
- **Tradutor de tela:** o OCR devolve linhas com retângulo e cores; cada parágrafo traduzido aparece no lugar. O fundo congelado é o screenshot (o `ScreencopyView` capturava a própria janela preta). Esc fecha: painéis `OnDemand` pegam o foco 50 ms depois de aparecer, como no Hyprland.
- **Conhost:** o profile troca a fonte da janela para JetBrainsMono NF, põe as 16 cores do ii na paleta (com fundo e texto do OSC 10/11 nos slots que o conhost usa como padrão), encaixa a janela na área útil e a deixa translúcida se a transparência do ii estiver ligada. Ícones do Oh My Posh trocados por equivalentes do plano 0. O magenta escuro fica invisível no próprio Windows PowerShell (divide o slot 5 com o fundo).
- **Crash do `VirtualDesktopAccessor.dll` do Win10** (`GetCurrentDesktopNumber` lendo nulo): chamadas protegidas por SEH; em falha, `RestartVirtualDesktopAccessor` uma vez por minuto, senão registro + atalhos.
- **Instância única:** atalhos, Run e lançamentos do setup usam `-n` (`--no-duplicate`).
- **Tema:** o `MaterialThemeLoader` tenta de novo quando lê o `colors.json` pela metade; o tema do terminal decide claro/escuro pelo próprio arquivo.
- **Rótulo:** "Swap" virou "WinPageFile" no Windows.
- **Barra de tarefas (achado no bare metal):** abrir um app mostrava a barra do Explorer por até 2 s (o timer de reesconder). Agora um WinEvent `EVENT_OBJECT_SHOW` no processo do Explorer reesconde na hora (no máximo 10 por segundo). Medido na VM: de "até 2 s" para no máximo ~20 ms em 3 aberturas.
- **Capa do álbum (achado no bare metal):** o `PlayerControl` copiava com `curl.exe` a capa que o backend já grava como `file:///`, e o cache do backend fixava "sem capa" para a faixa quando a capa chegava depois do título. As duas coisas foram corrigidas, e o worker relê as propriedades enquanto o título vier vazio. O Media Player do Windows 10 às vezes nunca publica o título (só a capa).
- **Bare metal do usuário:** pastas Área de Trabalho/Documentos/Imagens ficaram apontando para `OneDrive` depois que o WinUtil removeu o OneDrive (erro 362 no perfil do PowerShell). Problema da instalação; o setup ainda não detecta isso.
- **Comentários:** saíram do código; as explicações ficam em `notes/comments.md`, só local.

## 0.5.0 (2026-10-06)

- **Como foi feito:** dois workflows (1: visualizador, seletor de cor, WARP/game mode/avisos de desligamento, pastas conhecidas; 2: barra que encolhe e Super+arraste) com revisores; integração, testes e correções por mim nas VMs `win11` e `win10`.
- **Clima:** `XMLHttpRequest` para o wttr.in; sem GPS no Windows (evita o pedido de localização). Testado na win11.
- **Visualizador de mídia** sem cava (`AudioVisualizer`, loopback WASAPI + FFT) e **seletor de cor nativo** (Shift+Win+C, botão da barra, `/accentcolor`). Testados na win11.
- **Game mode, WARP (`warp-cli.exe`) e avisos de desligamento** (winget/msiexec/Windows Update, downloads incompletos): integrados, pouco testados.
- **Pastas conhecidas:** o setup avisa quando Documentos/Área de Trabalho/Imagens/Vídeos apontam para um OneDrive removido (erro 362), e o ii cai para outra pasta. `File::open` numa pasta dá erro 5 no Windows: o teste lista a pasta. Testado na win11.
- **Barra com escala alta:** mede em vez de usar os limiares fixos de largura. Primeiro reduz até 80%, depois esconde título e bandeja, por último reduz até 50%. Cada lado encolhe preso à própria borda (um `Scale` por seção, sem reindentar o código do upstream). Na win10 a 1920x1080/150% o 0.4.0 já punha a bandeja ~45 px por cima do relógio.
- **Gap no topo (amigo, Win10 22H2, TV a 150%):** numa troca de escala ao vivo o Explorer manda `TaskbarCreated`; o ii invalidava a AppBar, o `ABM_NEW` falhava (ela continuava registrada) e a barra ia para baixo da própria reserva antiga. Agora remove e registra de novo. Antes disso, o gancho da bandeja atrapalhava as chamadas de AppBar do próprio ii (`TrayHookYield`). Testado ao vivo a 100/125/150% na win10.
- **Papel de parede duplicado depois de trocar a escala:** janelas dentro do desktop não recebem `WM_DPICHANGED` e agora são recriadas; o zoom do papel de parede é recalculado quando a tela muda.
- **Papel de parede pelo Windows (pedido do usuário):** o papel de parede do ii surgia ~20 s depois do logon por cima do idêntico do Windows, como um app abrindo. Agora, por padrão, o Windows desenha o papel de parede, o ii o mantém igual nos dois sentidos (vigiando `HKCU\Control Panel\Desktop`; o `IDesktopWallpaper` não manda `WM_SETTINGCHANGE`) e só os widgets ficam no desktop, acima dos ícones. `windowsPort.ownWallpaper` volta o papel de parede do ii (parallax). Testado na win11.
- **Super+arraste** (`super_drag.cpp`): mover e redimensionar, com tiling (troca, divisória ao vivo, maximizada, flutuante, tile sozinho), entre monitores com escalas diferentes (monitor virtual do Virtual Display Driver na win10, removido depois), UWP; tela cheia e janelas de admin são ignoradas. Corrigido nos testes: subir a janela (`HWND_TOP` é ignorado fora do primeiro plano; agora `TOPMOST`/`NOTOPMOST`) e manter o ponto agarrado proporcional ao trocar de escala. Para testar input na VM, `SendInput` com `MOUSEEVENTF_ABSOLUTE`: um `SetCursorPos` + movimento zero não gera eventos no meio do arraste.
- **Fechar janelas de admin:** o `WM_CLOSE` é barrado pelo UIPI; cai para `WM_SYSCOMMAND/SC_CLOSE`. Uma janela que entra no tracker já em foco vira a ativa (Win+Q logo depois de abrir um app não fazia nada).
- **Limitação:** com uma janela de admin em foco o hook de teclado não recebe nada; o Win+Q vai para o Windows (abre a pesquisa) e o Win+Q seguinte fecha o que estiver na frente (fechou o PowerShell do usuário no teste). Está no README.
- **Agente da VM:** `ssh -n` nas chamadas que não leem stdin (o da win10 travou no fetch do `agent`).
- **Setup:** a limpeza depois de fechar (`cmd` + `ping` como espera) abria uma janela do Windows Terminal na win11: `CREATE_NO_WINDOW` é ignorado junto com `DETACHED_PROCESS`, e o `ping` criava um console próprio. Agora só `CREATE_NO_WINDOW`. Atualização 0.4.0 → 0.5.0 pelo pacote local testada na win11 (SHA-256, 0 avisos, ii reaberto).
- **Pendente:**
  - detecção de janelas no recorte de tela (não feita);
  - o `togglefloating` falhou uma vez e não repetiu;
  - o cheatsheet não cabe em 1024x768 (layout do upstream);
  - Super+arraste e a barra que encolhe ainda não foram vistos em hardware real.

## 0.10.1 "Tahiti Mangosellers" (2026-10-13)

Estável em cima da 0.10.0, publicada da branch `release-0.10` (detalhes no HANDOFF dessa branch): `Ctrl+Win+/` e o lápis do cheatsheet abrem um editor de atalhos (`modules/ii/keybindsEditor/`, issue #11: desligar devolve o atalho ao Windows, trocar teclas, restaurar, abrir o arquivo), mais as três correções guardadas. O mesmo editor está no `windows` do ii (fc225e98) para a próxima beta.

## 1.0.0 "Jazz-Handed Octopus" (2026-10-12)

Tudo o que faltava do end4-pC, nos dois visuais (inventário e plano em `build/inventory/`, notas por parte em `notes/comments.md`, seção "1.0.0 port"). Port feito por agentes em worktrees, integrado e testado por mim na `win10` e, para o que é só do Windows 11, na `win11-gpu`.
- Barra: estilos M3, M3 Hug, Panel e Split Hug, OLED, data no relógio, switchers, contador de updates (winget), avatar, Dock to Panel; dock com menu de clique direito; aba de mídia na lateral esquerda; menu de texto em todos os campos; equalizador sobre o Equalizer APO (apps de som dos fabricantes só abrem).
- Fundo: transições, prévia, centralizado, blur dividido, colagem (editor acima dos ícones com máscara só nos controles), soltar imagem no desktop (alvo OLE invisível acima do desktop, só imagem/vídeo; o resto o Explorer trata), provedores online, carrossel e submenus no menu do desktop, widgets novos.
- Página Sistema (Windows): telas (CCD, contagem de 15 s), entrada/cursor/animações, bordas e cantos (Windows 11), layout master e opções do dwindle, inatividade/energia (bloqueio desligado por padrão), autostart do Windows e do ii.
- Também: web apps, presets com zip/compartilhamento (via gh CLI), microfone na gravação (áudio do sistema ligado por padrão no Windows), regiões de conteúdo no recorte, velocidade de rede, checagem de atualização.
- Testado no Windows 11: bordas/cantos (e restauração ao sair), HDR liga/desliga, imagem da tela de bloqueio (funciona sem pacote; desliga o Spotlight, que foi restaurado). Sem teste em hardware: touchpad, microfone, equalizador com APO.
- Setup 1.0.0 com pacote local instalado na `win11-gpu` (0.9.2.2 → 1.0.0). A `win10` tem o build de dev da 1.0.
- **1.0.0-beta.2:** crash ao desligar/remover o monitor (issue #10: a janela rastreada guardava o QScreen removido e o tiling usava na próxima troca de foco; agora esquece a tela no `screenRemoved`). Menos RAM: uma janela escondida há 10 s libera o device D3D11 e a swap chain (cada painel aberto uma vez prendia ~23 MB na VM sem GPU; reabrir custa ~20 ms a mais), o cheatsheet descarrega 2 min depois de fechado, `gc()` depois de descarregar painéis, segment heap no manifest. Na `win11` sem GPU, depois de abrir e fechar tudo: ~296 → ~195 MB privados.
- **1.0.0-beta.3:** pílula "Now Playing" (port do que foi feito primeiro no end4-pC do Fedora): na troca de música a capa sai do widget de mídia da barra, voa até a pílula flutuante (arrastável, posição em `Persistent.states.nowPlaying`), mostra título, artistas e álbum e volta; o fundo do widget na barra some enquanto isso. Velocidades, tamanho, tempo e o voo em Configurações → Barra → Now Playing (nos dois layouts). No Windows não aparece se o monitor tem uma janela em tela cheia (a camada Top vira topmost). Checagem de atualização em toda inicialização (e a cada 24 h), notificação com o ícone do ii-windows e os botões Atualizar/Novidades (Atualizar abre a página de atualização do setup; nunca instala sozinho); instalações beta veem betas novas. Testado na `win10` (troca de faixa com mp3 mudos no Reprodutor Multimídia; config restaurada por backup byte a byte).
- **1.0.0-beta.4:** as correções da 0.10.1-fix3 e da fix4 (barra lateral sem páginas fantasmas, paleta de presets, nome da galeria, menu das configurações minimal, card de mídia da dock, sem hyprctl, fonte do sistema no lugar da embutida, "special" em monitor novo). Aba "Uso de IA" no cheatsheet (`services/AiLimits.qml`: limites do Claude e do Codex pelos logins locais, sem renovar token; uso da semana por modelo das transcrições do Claude Code via `scripts/ai/claude-local-usage.{py,ps1}`; liga/desliga e intervalo em Interface > Cheat sheet). No motor: hot reload sem laço de recarregar atalhos/blur, e o arquivo de usuário dos atalhos e do blur é declarado pela config (`"userFile"` no `defaults/windows/*.json`; o ii usa `illogical-impulse/...`, uma config qualquer não herda mais o do ii).
- **1.0.0 estável (2026-10-14):** a beta.4 mais a sensação de 60 Hz num monitor de 200 Hz (o Qt avança as animações por um timer do tamanho do quadro da tela *primária* quando há mais de uma janela exposta, e o HDMI fantasma de 60 Hz pode ser a primária): `src/windows/animation_timing.cpp` instala um contexto de scene graph próprio (`qswindows`) com timer pela tela mais rápida, relógio de parede, ~30 passos/s quando só janelas escondidas animam, re-tempo quando a taxa muda; `QS_WINDOWS_ANIMATION_HZ` limita, 0 desliga. Anexos do Gemini no Windows (`scripts/ai/gemini-upload.ps1`) e colar imagem no chat. Revisão multi-agente (5 áreas + verificadores, 35 achados confirmados): animações invisíveis paradas (dial de volume, barras onduladas, relógio girando, visualizador), atalhos Win enviados pelo shell marcados para o próprio hook, sem travar em janela travada/worker preso, troca de layout de teclado barata, dock por monitor (`dock.screenList`), VPN e Gemini/AI usage. Detalhes em notes/comments.md (local). Depois, também na 1.0.0: o editor de atalhos grava só as mudanças (`"extends": "defaults"` + `{"base": id, "keys"|"disabled"}`, id = ação:argumento com @teclas quando repetido; o hotkeys.cpp junta com os padrões; arquivo antigo completo é convertido na próxima edição), a janela do Now Playing cobre só a pílula e o voo (tela cheia só depois de passar o mouse/clicar, para arrastar), desktops novos são criados por timer em vez de msleep no thread da GUI, e exec de atalho/dispatcher roda num pool próprio (`runLauncher`, COM STA). Testado na win11-gpu (200 Hz, RTX): backend qswindows + D3D11, 200 ticks/s visível, ~29/s só em janela escondida, timer de animação parado em repouso, setup beta → 1.0.0 offline, editor esparso e conversão de arquivo antigo, Win+0 criando o 10º desktop, exec de atalho, Shift+Win+Seta sem laço.
- **1.0.1 (2026-10-14):** revisão de CPU e RAM depois da 1.0. CPU: tiling re-tenta janela travada a cada 500 ms sem prender a GUI; gravação reserva um arquivo único antes de começar; o histórico do clipboard não recaptura a imagem copiada dele (a sequência é lida depois de fechar o clipboard); indicador de carregamento e marcador do mapa param quando ninguém vê; o visualizador do desktop pausa com o desktop coberto, bloqueado ou em modo jogo (também com os widgets acima dos ícones, onde o relógio girando estava parado); o cronômetro só atualiza enquanto aparece (1 Hz na ilha e no card do desktop, 100 Hz só na sub-aba visível da barra lateral); notificações descartadas são liberadas. RAM: mensagens da IA removidas, limpas ou trocadas por /load são destruídas quando nenhuma resposta, upload ou comando escreve mais nelas; miniaturas de wallpaper online decodificadas no tamanho da célula; a capa desfocada da mídia só é carregada com o blur na tela, em 128 px; as abas da barra lateral esquerda carregam quando visitadas e saem 120 s depois de paradas, guardando rascunho, tradução e rolagem; o Overview é liberado 2 min depois de fechado; capturas de tela saem da memória quando o seletor fecha; o preview do dock solta a captura depois do fade. /save sempre grava e /load espera o arquivo. Testado na `win10` (números em notes/comments.md, local).
- **1.0.2 (2026-10-14):** issue #15. O atalho "illogical-impulse Settings" do Menu Iniciar (`qsw -n -p settings.qml`) abria as configurações num processo separado: ali "Escolher arquivo" não alcançava o seletor de wallpaper e Claro/Escuro trocava o Windows e as cores do shell, mas a janela não se atualizava. No Windows o `settings.qml` agora pede ao shell `ipc call settings open` e sai; só abre a janela própria se o shell não estiver rodando (ou não responder em 4 s). Testado na `win10`: ~330 ms com o shell, ~610 ms sem ele.
- **1.0.3 (2026-10-15):** issue #16 (texto e ícones viravam caixinhas ▯ ou glifos errados, em todas as janelas, até o cache de fontes soltar). O FreeType do Qt6Gui.dll abre cada face de fonte vinda de arquivo com `fopen` e mantém o `FILE*` aberto enquanto o engine está em cache; cada combinação de eixos variáveis é um face, e glifo faltando faz o Qt percorrer as fontes do sistema. Com muitas fontes instaladas, ou com uma fonte que o ii varia instalada no sistema, passava do limite padrão de 512 streams do CRT e todo `FT_New_Face` novo falhava ("QFontEngineFT: Failed to create FreeType font engine", 163 mil vezes no log do relato). O `qsw` agora chama `_setmaxstdio(8192)` no início. Testado na `win10`: 800 textos em Bahnschrift com pesos diferentes davam 1.082.547 falhas a partir de ~250 variações no binário antigo; com o novo, zero (Bahnschrift e Google Sans Flex).

## 0.10.0 "Dancing Lighthouse" (2026-10-12)

- **Papel de parede animado:** um vídeo (MP4, WebM, MKV, AVI, MOV) escolhido como qualquer imagem toca atrás dos ícones do desktop. Media Foundation (IMFMediaEngine em modo frame server) numa thread própria, com o próprio device D3D11, escreve cada quadro numa textura D3D11 compartilhada que o Qt copia; um decodificador por arquivo para todos os monitores. As cores vêm de um quadro extraído (também usado como papel de parede do Windows, banner da barra lateral e tema do terminal). Pausa quando uma janela maximizada ou em tela cheia cobre o monitor (`background.pauseVideoWhenCovered`), com a tela bloqueada e no modo jogo. Pasta de vídeos no menu do desktop ("Live wallpaper...", `wallpaperSelector.liveWallpapersPath`) e miniaturas com selo de play no seletor.
- **Testado** na `win11-gpu` (RTX: decodificação na GPU, ~0,3 s de CPU a cada 4 s, zero quando coberto) e na `win10` sem GPU (dois monitores; cai para 720p/15 fps: ~50–90% de um núcleo). Trocar de vídeo para vídeo derrubava o shell (liberação dupla da textura do Qt); corrigido e testado com trocas seguidas entre H.264 1080p/720p e VP9.

## 0.9.2.2 "Melomaniac Aviator" (2026-10-12)

Correções do que apareceu gravando o Reel (`notes/reel-findings-2026-10-11.md`), testadas na `win11-gpu` (RTX 5060 Ti, 1920x1080) e instaladas lá pelo setup novo (0.9.2.1 → 0.9.2.2, pacote local).

- **Travamento com dock lateral:** aplicar um preset com a dock à esquerda/direita e ícone 33/34 travava o shell. A janela da dock e os filhos discordavam da orientação por um instante e a largura crescia sem parar (laço de polish). A janela agora segue o `DockStyle`, e a máscara de input de um painel fica limitada à própria janela. Testado aplicando o Everforest e reiniciando com a dock lateral.
- **Emoji:** resultado exato primeiro e sem bandeiras no Windows (a Segoe UI Emoji não tem bandeiras); a grade do Spotlight não rola mais sozinha quando a lista muda.
- **Layouts da barra:** trocar o estilo guarda o layout personalizado de cada estilo em vez de apagar.
- **Configurações:** trocar o estilo com as Configurações abertas não fecha e reabre a janela (a nova aparece antes de a antiga sair).
- **Seletor de papel de parede:** o lápis do caminho põe o foco no campo, e o texto não passa mais por cima do ícone.
- **Widgets:** um widget ligado com o shell aberto vai para um espaço livre; Notas, Tarefas, Texto e Relógio mundial pegam o teclado ao entrar em edição; clicar no card do timer inicia/pausa.
- **Barra:** mostra "Desktop" quando nada está em foco no monitor.
- **Tiling:** Shift+Super+seta numa janela flutuante a leva para a borda da tela, sem o encaixe nativo.
- **Atalhos (cheatsheet):** Esc fecha.
- **Tooltips:** voltam a ser desenhados acima do resto da janela (desde a 0.6.2 ficavam presos ao botão).
- **Opções:** um só switch para o card de mídia da barra lateral; Spicy Stuff saiu da página inicial das Configurações; "Esconder o nome do computador" (barra lateral e aba Sistema).
- **Setup:** atualização, reparo e desinstalação não pedem mais para ler o readme; o aviso de versão nova tem linha própria; avisa quando o ii fecha logo depois de abrir.
- **Ctrl+Super+P:** medido, sem mudança: o Explorer aplica o auto-ocultar ~300 ms depois da chamada e o resto é criar/destruir a barra do ii.
- **Ferramentas:** fila de jobs por VM (`IIW_VM`) e jobs com BOM.

## 0.9.2.1 "Sleepwalking Toaster" (2026-10-11)

- **Versão com quatro partes:** o instalador compara versões parte a parte (`installer/core/src/version.rs`), então 0.9.2.1 é mais nova que 0.9.2 e a autoatualização funciona. O Cargo só aceita três partes: `installer/Cargo.toml` fica em 0.9.2 e o `tools/release.sh` aceita isso quando o `VERSION` tem uma quarta parte; o setup mostra o `VERSION` (via `build.rs`).
- **Painéis seguem o mouse (issue #5, de novo):** o monitor focado do shim do Hyprland acompanhava também a janela ativa; ao fechar um painel o Windows devolve o primeiro plano para alguma janela, quase sempre no monitor principal, e o painel seguinte abria lá mesmo com o mouse no outro monitor. Agora o monitor focado é o do cursor (como o follow_mouse do Hyprland): o hook do mouse avisa quando o cursor muda de monitor, e os painéis sem tela fixa abrem no monitor do cursor.
- **Cartão de Configurações com X (issue #7, de novo):** o cartão do end4-pC não tinha botão de fechar; com "Manter aberto ao clicar fora" ligado ele só fechava com Esc, e só com o teclado nele.
- **Gravação com dois monitores (issue #8):** ao começar a gravar, os seletores dos outros monitores agora fecham.
- **Capa um música atrás (issue #9):** o Spotify (via GSMTC) atualiza o título antes da capa; a capa lida logo depois da troca era a da música anterior e ficava salva no cache com o nome da música nova. Agora o cache é pelo conteúdo da imagem, a capa é relida a cada mudança e uma capa igual à da música anterior é relida algumas vezes antes de ser aceita.
- **Testado na `win10`** com dois monitores (monitor virtual a 150%) e três músicas de teste com capas diferentes no Reprodutor Multimídia; o caso exato do Spotify (título antes da capa) não acontece com ele.

## 0.9.2 "Caffeinated Pigeon" (2026-10-10)

- **Texto (issue #6):** no Windows o Qt desenhava com DirectWrite e ClearType (subpixel): franjas coloridas nos painéis escuros e translúcidos e dígitos irregulares em escala fracionária (150%). Agora `QT_QPA_PLATFORM` tem padrão `windows:fontengine=freetype` (`launch.cpp`; um valor definido pelo usuário vence): tons de cinza, como no Linux. Emoji colorido e fallback CJK conferidos na `win10`.
- **Configurações abertas (issue #7):** opção `settings.keepOpen` ("Manter aberto ao clicar fora"), nos dois layouts. O cartão do end4-pC deixa de ser dispensável no `GlobalFocusGrab` e, no Windows, usa foco OnDemand (um painel Exclusive retoma o primeiro plano a cada desativação) com uma captura momentânea para pegar o teclado ao abrir. Os estilos ii e Dashboard são janelas normais e nunca fechavam sozinhos.

## 0.9.1 "Haunted Spatula" (2026-10-10)

- **Ícones de app (issue #2):** no Windows o `AppSearch.guessIcon` descartava os ícones dos apps (`appicon:`), porque `Quickshell.iconPath(nome, true)` só conhecia o tema; janelas sem AUMID próprio (explorer, mspaint, cmd, powershell…) caíam na engrenagem genérica no dock, na barra, nos workspaces e no overview. Agora a varredura do menu guarda o .exe de destino de cada app (`System.Link.TargetParsingPath`), `DesktopEntries.byId` acha o app pelo .exe da janela (caminho completo, depois só o nome) e janelas sem app no menu usam o ícone do próprio .exe.
- **Fixados padrão:** Explorador de Arquivos e Terminal do Windows (PowerShell onde o Terminal não existe) no lugar de Dolphin e kitty, no dock e no menu Iniciar do estilo waffle; configs antigas trocam uma vez (`windowsPort.pinsVersion`, `services/WindowsPins.qml`).
- **Contorno da barra (issue #3):** com "Mostrar fundo" desligado o estilo flutuante ainda desenhava a borda de 1 px (igual no upstream).
- **Painéis no monitor focado (issue #5):** painéis sem tela fixa (barras laterais, overview, cheatsheet…) abriam sempre no monitor principal, porque no Windows não há compositor escolhendo a saída. Agora abrem no monitor focado, e um clique em qualquer monitor (barra, área de trabalho ou janela) passa a focá-lo, como o foco que segue o mouse no Hyprland.
- **Captura no segundo monitor (issue #4):** o seletor de região de cada monitor capturava a tela durante a criação, antes de receber o próprio monitor; todos capturavam o principal e o arquivo do segundo nunca existia, então o recorte não ia para a área de transferência. A captura agora roda depois da criação (`TempScreenshotProcess.captureWindows`).
- **Atalhos (issue #1):** era um mod do Windhawk interceptando os atalhos com Super; nada mudou no ii.
- **Testado com dois monitores** na `win10` (monitor virtual do Virtual Display Driver, 1920x1080 a 150% à direita do principal): painéis seguem o clique, recorte no segundo monitor copia a imagem certa.
- **Pendente:** monitor ligado com o ii aberto mostra "special" na área de trabalho da barra nova até reiniciar; monitor à esquerda do principal (coordenadas negativas) não testado.

## 0.9.0 "Exploding Baby" (2026-10-09)

- **Nome:** a partir desta versão toda release tem um nome aleatório e engraçado (`RELEASE_NAME` na raiz; o instalador mostra "0.9.0 · Exploding Baby" na barra de título). As anteriores são "Unnamed". A 0.8.0 planejada foi junto: não existe 0.8.0.
- **Port do end4-pC (etapa 2), nos dois estilos e nos dois layouts de Configurações (449 opções, `tools/settings-parity.py`):** barra configurável (motor de layout do fork só quando o layout difere do clássico; o padrão carrega o `BarContent` antigo), ilha dinâmica, moldura de tela com AppBars, medidores de RAM/disco, letras sincronizadas (lrclib), uso do Claude Code; dock em qualquer borda; 13 widgets de área de trabalho com grade/snap/trava; Wallhaven, ordenação, troca automática e pastas Wpp no seletor; banner, cartão de mídia, seções reordenáveis, cantos ativos e VPN na barra lateral direita; estilos mínimo e painel, perfil e predefinições nas Configurações; posição das notificações, overview estilo niri, prefixos de busca, clipboard fixado, esquemas de cor nomeados, rolagem inercial, bateria de periféricos Bluetooth, menu da área de trabalho e prateleira de arquivos. O estilo end4-pC também aplica o layout de barra e o banner do fork (`VisualStyle.presetVersion`: quem já usava recebe só as chaves novas, uma vez).
- **Ctrl+Super+P:** troca a barra do ii pela barra de tarefas do Windows (widgets e painéis ficam).
- **Spicy Stuff:** Konachan (filtros de qualidade, só yuri, tags extras), booru, Wallhaven e a pasta Wpp (Spicy) passam por `services/SpicyStuff.qml`. No Windows exige conta Microsoft que o Windows confirma ter 18+ (`AccountAge` em C++: `NetUserGetInfo` nível 24 + `CheckUserAgeConsentGroupAsync(Adult)`, só existe no Windows 11); conta local, menor de idade, idade não confirmada ou Windows 10 ficam bloqueados com o motivo.
- **Como foi feito:** seis agentes de port em worktrees (barra Opus, resto Sonnet), revisão por port, merge e testes por mim na `win10`, depois uma revisão agressiva com seis revisores (segurança e integração Opus, desempenho/C++/padrões Sonnet, verificações mecânicas Haiku). Ferramentas novas: `tools/qml-compile-check.py` (compila todo QML na VM, pega erros de arquivos carregados sob demanda) e `tools/qml-import-check.py` (singleton sem import e pasta sem qmldir). Detalhes em `notes/comments.md` (local).
- **Corrigido na revisão:** `ClippingRectangle` não desenhava nada no Windows (shaders sem HLSL no upstream); fuso do relógio mundial (V4 ignora `timeZone`, agora `QSWin.TimeZones`); injeção de parâmetros no Wallhaven (purity), predefinições limitadas a chaves visuais (sem comandos, caminhos UNC, dados pessoais), nomes de arquivo vindos das APIs validados, guarda da pasta Spicy com caminho canônico; botão de bloqueio deixava o ii "bloqueado" depois de desbloquear o Windows; atalhos novos chegam a quem tem `keybinds.json` próprio (`since` no padrão).
- **Inicialização:** igual à 0.7.0 na VM limitada (barra reservada em ~2,9 s; 0.7.0 3,0–3,7 s no mesmo A/B); +0,9 s de CPU e +15 MB no início. Pacote 195 MB (zip 80 MB).
- **Pendente:** foco de teclado dos widgets é uma flag única; ordem das AppBars depois de alternar o modo barra de tarefas; buraco na moldura quando a barra some; chave do Wallhaven em texto no config.json; script do uso do Claude relê os transcripts a cada minuto (opcional); idade da conta Microsoft só testada no caminho local (VM) — falta hardware real com conta Microsoft 18+.

## 0.7.0 (2026-10-09)

- **Estilo visual (etapa 1):** escolha entre o visual original do ii e o do fork end4-pC (pctrade), no instalador (instalar e atualizar, com uma imagem de cada) e em Configurações > Básico > Estilo. Os dois têm as mesmas funções e opções: o layout de Configurações do end4-pC (`modules/settingsPc`, cartão flutuante com trilho, avatar e busca pelo Spotlight/Overview) expõe exatamente as mesmas 277 opções do layout original (`tools/settings-parity.py` confere). O estilo end4-pC também liga números nas áreas de trabalho, seletor de cor na barra e controles rápidos na barra lateral (`services/VisualStyle.qml`); o instalador só grava `appearance.visualStyle` e o ii aplica o resto ao abrir. As funções exclusivas do fork ficam para as próximas etapas (`notes/end4pc-fork-only-keys.txt`, local).
- **Segunda revisão agressiva** (mesma VM limitada da 0.6.2, A/B intercalado com o build anterior, 5 inicializações cada): configuração carregada 0,97 → 0,84 s e primeiro quadro da barra 1,79 → 1,63 s (mediana). Varredura do QML e bundle numa thread em paralelo com as fontes, varredura sem consultar o disco várias vezes por pasta, fonte Rubik (nunca usada, 14 das 49 faces) fora do pacote.
- **Memória:** 265 → 239 MB privados 40 s depois de abrir (cheatsheet sem pré-carregamento aos 20 s; continua mantido depois da primeira abertura). Seletor de papel de parede e menu de sessão liberam a janela depois de 2 min fechados (−29 MB no seletor). JetBrains Mono NF: só o Regular quando a instalação por usuário está incompleta (eram 38 MB em memória).
- **Pacote:** 208 → 182 MB (sem `opengl32sw.dll`, sem o `d3dcompiler_47.dll` que o Windows já tem, sem estilos Imagine/Universal, plugins de posição e Rubik).
- **Corrigido:** depois de um kill, crash ou atualização a barra aparecia 45 px abaixo e pulava ~2 s depois (AppBar fantasma da instância morta); agora os registros ficam em `HKCU\Software\Quickshell\AppBars` e os de janelas mortas são removidos antes de posicionar. "Dots" no mostrador do relógio traduzido como "Pontos".
- **Pendente:** duas instâncias ao mesmo tempo e mudança de DPI com a AppBar nova não testadas; carga dos tipos QML (~200 ms) e fontes (160–280 ms) são as próximas pistas de inicialização.

## 0.6.2 (2026-10-08)

- **Como foi feito:** revisão do código todo por sete agentes (caminho de inicialização, backend Windows, janelas/painéis, QML de inicialização, módulos do ii, app de Configurações, deploy/Qt), medição nas VMs e implementação em seis branches paralelos (widgets e barra, serviços, Configurações, painéis, eventos C++, render/IO C++), com revisão, integração e testes por mim na `win10` e na `win11`.
- **Inicialização em PC fraco** (VM com 2 vCPUs de E-core a 40% e disco a 120 IOPS / 30 MB/s): primeiro quadro da barra em ~2,0 s na win10 (0.6.1: ~5,3 s) e ~2,1 s na win11; configuração carregada em ~1,0 s (era 3,4 s).
  - Cache do scanner de QML (`qmlscan.bin`): arquivo com tamanho e data iguais não é aberto; 2,2 s → 0,15 s. O deploy e o instalador agora preservam as datas dos arquivos, então o cache sobrevive às atualizações.
  - Portão "depois do primeiro quadro": DLL dos desktops virtuais, lista inicial de janelas, reserva da AppBar, Wi-Fi/rede, mídia, Bluetooth, dispositivos de áudio extras, bandeja e compositor do blur do Win11 esperam a barra aparecer.
  - Logs detalhados desligados por padrão no Windows (`QS_DETAILED_LOGS=1` religa); fontes Material Symbols e Google Sans Flex sem o eixo GRAD (14,6 → 8,1 MB e 4,0 → 2,2 MB); `qsw.exe` com `/Ob2 /Gw /OPT:REF,ICF` (6,5 → 5,0 MB); pacote sem FFmpeg, Qt Multimedia, Designer e afins (251 → 206 MB); limpeza automática de instâncias antigas em `run\` (189 instâncias e 70 MB na VM).
  - QML: texto, botões e dicas não criam mais animações e camadas de efeito que nunca rodam; popups da barra montados ao abrir; serviços do Linux e pastas que nem existem no Windows não são mais carregados na inicialização; tema e tradução chegam antes da barra (sem recolorir e retraduzir tudo).
- **Configurações:** abrem dentro do próprio shell no Windows (sem processo novo): 0,6–1,3 s já com tema e em português no primeiro quadro (antes: janela vazia em ~1 s, branca e em inglês, pronta em ~2,5–3 s). O `qsw -p settings.qml` continua funcionando (atalho do Menu Iniciar) e também abre já pronto, sem o quadro branco.
- **Painéis:** sessão, OSD, controles de mídia, seletor de papel de parede e cheatsheet mantêm a janela depois da primeira abertura; sidebar esquerda e cheatsheet pré-carregados em segundo plano; spotlight reaproveita os itens ao digitar; foco de teclado na primeira exposição (digitar logo depois do Win cai na busca).
- **Mouse em máquinas mais fracas:** o hook de mouse decide sozinho quando uma máscara precisa mudar e só acorda a thread da interface nesse caso; eventos de janelas irrelevantes são descartados antes de chegar nela.
- **Corrigido:** o brilho era regravado no monitor a cada inicialização (DDC ia a 100%); tradução não carregava com idioma "auto"; Super+Q com a janela de Configurações em foco fechava o app anterior; texto sobreposto no cheatsheet; seletor de papel de parede cortado em telas pequenas.
- **Mudanças de comportamento:** com "barra de tarefas só ao passar o mouse", o ocultar automático do Windows continua ligado depois de fechar o ii (o desinstalador restaura); no Windows algumas máscaras de painel inteiro saíram (bordas cortadas retas em alguns painéis, miniaturas do overview sem cantos arredondados).
- **Pendente:**
  - a primeira inicialização depois de instalar arquivos novos continua lenta (Defender e disco frio em cada DLL regravada), uma vez por atualização;
  - blur do Win11 com GPU de verdade e brilho por WMI em notebook ainda não vistos em hardware real.

## 0.6.1 (2026-10-07)

- **Configurações completas:** das 304 opções do `config.json`, 118 não tinham controle; ~90 entraram no app (Geral, Barra, Fundo, Interface, Serviços), em quatro agentes paralelos com revisão e integração por mim. O que só vale no Linux fica escondido no Windows. Ficaram só no arquivo: posições arrastadas de widgets, estado salvo do game mode, listas que o próprio ii edita (apps fixados, bandeja, botões rápidos) e `ai.extraModels`. Opções mortas (sem uso no código) não ganharam controle.
- **Corrigido nos testes (win10):** dicas presas na tela (os `ConfigSpinBox`/`ConfigSlider` não tinham `hovered`), campos de lista que comiam a vírgula ao digitar (agora gravam ao sair do campo), cor de destaque que só valia na próxima troca de papel de parede, linhas apertadas a 1024 px, botão do arquivo de config cortado em pt_BR.
- **pt_BR:** 196 textos das Configurações traduzidos.

## 0.6.0 (2026-10-07)

- **Como foi feito:** dois workflows/rodadas de agentes (otimização: posicionamento dos widgets, inicialização nativa, pool de threads, sidebars, painéis, construtores dos serviços, cache de bytecode; mais o índice de arquivos), com revisão, integração e testes por mim nas VMs `win10`, `win11` e `win11-gpu`.
- **Inicialização em PC fraco** (VM presa a 2 E-cores a 40% e disco a 120 IOPS / 30 MB/s): barra em ~4 s (0.5.0: ~43 s), CPU de ~40 s para ~8 s. Sem limite: ~1 s.
  - Scanner de QML: `QVector<QDir>::contains` chamava `canonicalPath` dos dois lados a cada diretório (34,6 s de 38 s); agora um `QSet` de caminhos canônicos.
  - Painéis: a família só compila os painéis que cria; no Windows a barra primeiro e o resto um por volta do event loop; sem watcher de QML no Windows (`IIW_WATCH_FILES=1` liga).
  - Cache de bytecode QML gerado no deploy e servido pelo hook de unidades em cache do Qt quando o hash do fonte bate (o esquema `qs:` não passa pelo cache em disco do Qt).
  - Tirados da thread da interface: posição "least busy" dos widgets (thread própria + cache), espera pelo WorkerW do Explorer, fontes já instaladas, varredura de apps/brilho/miniaturas (pool próprio), Wi-Fi/rede/teclado/OCR/estatísticas, o broadcast `WM_SETTINGCHANGE` da cor de destaque e os watchers do Menu Iniciar (767 ms → 9 ms).
- **Aba System no cheatsheet** e **widget de CPU/RAM/disco no desktop** (a partir de um post do canal de Tweaks do Discord do end-4; implementação nossa). Ajusta a 3/2/1 colunas em telas pequenas (testado em 1024x768 e 125%).
- **Busca Spotlight** (a partir de vídeos do mesmo canal; implementação nossa): Win sozinho abre a busca (apps, arquivos, área de transferência, emoji, ações do sistema, web); Win+Tab abre os workspaces sem a busca. Toggle em Configurações > Services > Search. Arquivos no Windows por um índice em memória da pasta do usuário (`FileIndex`), placeholders do OneDrive incluídos.
- **Emoji no Windows:** a lista vinha do script do Hyprland, que não existe no Windows; agora vai em `defaults/windows/fuzzel-emoji.sh` (Win+. e o spotlight).
- **Entrada atrasada em máquinas mais fracas** (Windows 11, mouse e teclado com lag): as threads dos hooks de mouse e teclado saem do EcoQoS do Windows 11 (o qsw não tem janela em primeiro plano e pode ser tratado como processo de fundo) e rodam com prioridade crítica; o processo sai do throttling de velocidade. Nas VMs não reproduz (nenhum evento chegou atrasado nos hooks); a linha "Input reached the hooks late" do log diz se ainda acontece.
- **Crashes ao recarregar:** handler do UPower e retornos das threads do brilho e do Wi-Fi seguravam o singleton morto; agora `QPointer`.
- **Pendente:**
  - a cor de destaque não fica no Windows 10: o Explorer reescreve `DWM\AccentColor` a partir de `Explorer\Accent` depois do broadcast (precisaria gravar `AccentColorMenu`/`AccentPalette`);
  - a aba System ainda não foi vista com uma GPU de verdade (a NVIDIA da `win11-gpu`);
  - atalho de um painel ainda não criado no primeiro segundo é ignorado.

## A fazer

- **Assinatura de código (decidido em 2026-10-06):** adiada. Se o número de usuários crescer, o caminho é a SignPath Foundation (gratuita; exige build verificável no GitHub Actions, página de política de assinatura com papéis e declaração de privacidade). Nenhuma opção cala o SmartScreen de imediato desde 2024.
