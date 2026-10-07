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

## A fazer (2026-10-07)

- **Aba "System" no cheatsheet** (pedido do usuário, com um screenshot de um ii no CachyOS: abas Keybinds | System | AI usage; o nosso tem Keybinds | Elements). Não está no `end-4/dots-hyprland` até 03/10 (o PR #1858 "More detailed resources usage" é parecido, mas é outro): descobrir de onde vem antes de portar (licença, e juntar em vez de reescrever).
  - Cabeçalho: host, sistema, kernel, placa-mãe, tempo ligado.
  - CPU: modelo, núcleos/threads, clock máximo, L3; uso, temperatura e clock em medidores; gráfico de uso.
  - GPU: modelo, VRAM, driver/VBIOS; uso, temperatura (e hotspot), VRAM usada; potência, clock, temperatura da VRAM, ventoinha; gráfico de uso.
  - Memória: RAM, swap (no Windows, o arquivo de paginação) e VRAM em barras.
  - Armazenamento: discos (modelo, tipo, tamanho, temperatura) e partições com sistema de arquivos e uso.
  - Aba "AI usage" (conteúdo não aparece no screenshot) e cards de CPU/RAM/disco no desktop, ao lado do relógio.
  - Fontes no Windows a avaliar: `Win32_BaseBoard`/`Win32_Processor` (WMI), contadores PDH "GPU Engine" e DXGI `QueryVideoMemoryInfo` para a GPU, temperatura/potência/ventoinha só por API do fabricante (NVML, ADLX), temperatura de disco por `MSFT_StorageReliabilityCounter`; temperatura de CPU não tem API confiável sem driver.
- **Assinatura de código (decidido em 2026-10-06):** adiada. Se o número de usuários crescer, o caminho é a SignPath Foundation (gratuita; exige build verificável no GitHub Actions, página de política de assinatura com papéis e declaração de privacidade). Nenhuma opção cala o SmartScreen de imediato desde 2024.

