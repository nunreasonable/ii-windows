# Antes de instalar

O **illogical-impulse** (ii) é o shell de desktop do end-4 para o Hyprland no Linux: barra,
painéis laterais, visão geral, lançador, notificações e cores Material You tiradas do papel de
parede. O **ii-windows** é um port não oficial dele para o Windows 11, rodando numa versão do
Quickshell para Windows. É experimental: espere arestas.

Esta página lista o que este instalador muda no seu computador e o que o ii muda enquanto
roda. Tudo é instalado só para o seu usuário do Windows; não é preciso permissão de
administrador, a não ser para o PowerShell 7 opcional.

## O que é instalado, e onde

- **O programa** vai para `%LOCALAPPDATA%\ii-windows` (cerca de 240 MB): o Quickshell
  (`qs.exe`, `qsw.exe`), as bibliotecas do Qt, as fontes e os ícones que o ii usa, o
  `matugen.exe` (gera a paleta de cores a partir do papel de parede) e a
  `VirtualDesktopAccessor.dll`. Uma cópia deste instalador também fica lá, junto com o
  `install-manifest.json` (o registro de tudo que este instalador mudou, usado para desfazer) e o
  `setup.log`.
- **Os arquivos do próprio ii** vão para `%LOCALAPPDATA%\quickshell\ii`. Eles são substituídos a
  cada instalação, atualização e reparo, então não os edite.
- **As suas configurações** ficam em `%LOCALAPPDATA%\illogical-impulse` (`config.json` e
  outros). O ii cria esses arquivos conforme você muda as coisas; instalar e atualizar nunca
  mexem neles.
- **O estado e o cache do ii** (cores atuais, lista de tarefas, histórico de notificações, logs,
  relatórios de falha) vão para `%LOCALAPPDATA%\quickshell` (`State`, `cache`, `run`), e os
  arquivos temporários para `%TEMP%\quickshell`. Uma paleta de cores padrão é colocada se ainda
  não houver nenhuma.
- **Menu Iniciar:** "illogical-impulse" (abre o ii) e "illogical-impulse Settings".
- **Aplicativos instalados:** uma entrada chamada "illogical-impulse (ii-windows)". *Modificar*
  abre este instalador (atualizar, reparar, desinstalar) e *Desinstalar* o abre na página de
  desinstalação.
- **O download:** o pacote vem da última versão publicada de
  [nunreasonable/ii-windows](https://github.com/nunreasonable/ii-windows) no GitHub e é conferido
  com o checksum SHA-256 publicado junto antes de qualquer coisa ser instalada.

Antes da primeira instalação, o instalador também anota como o Windows está agora, para que a
desinstalação possa voltar a isso: o papel de parede (uma cópia da imagem fica em
`%LOCALAPPDATA%\ii-windows\restore`), o modo claro ou escuro, a cor de destaque e se a barra de
tarefas se oculta automaticamente.

## As opções

- **Iniciar com o Windows** (ligada por padrão): adiciona `illogical-impulse` à lista "Run" do
  seu usuário (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`), para o ii abrir quando você
  entra no Windows.
- **Configuração do terminal** (ligada por padrão), a versão Windows do visual de terminal do ii:
  - a fonte *JetBrainsMono Nerd Font* é instalada para o seu usuário (copiada para
    `%LOCALAPPDATA%\Microsoft\Windows\Fonts` e registrada em `HKCU`); arquivos de fonte que você
    já tem não são mexidos;
  - o *Oh My Posh*, o *Starship* e o *eza* são instalados com `winget install --scope user`, a
    não ser que já estejam lá. O winget roda com `--accept-package-agreements` e
    `--accept-source-agreements`, ou seja, este instalador aceita por você as licenças desses
    pacotes e os termos da fonte do winget. Só as ferramentas que este instalador instalou são
    lembradas para a desinstalação;
  - um bloco marcado é adicionado ao seu perfil do PowerShell (o `$PROFILE` do Windows PowerShell
    5.1, e o do PowerShell 7 se ele estiver instalado). Ele carrega o prompt do ii, as cores do
    terminal e alguns aliases (`ls` lista arquivos com o eza, `clear`, e `q` abre o ii). O perfil
    é copiado para um backup antes de ser editado, e o bloco não faz nada depois que o ii é
    removido:

    ```
    # >>> illogical-impulse >>>
    # Loads illogical-impulse's terminal setup (prompt, colors, aliases). Remove this block, markers
    # included, to turn it off; the ii installer removes it on uninstall.
    $IiProfile = Join-Path $env:LOCALAPPDATA 'quickshell\ii\defaults\windows\terminal\profile.ps1'
    if (Test-Path -LiteralPath $IiProfile) { . $IiProfile }
    # <<< illogical-impulse <<<
    ```
- **Instalar o PowerShell 7** (desligada por padrão): `winget install Microsoft.PowerShell`. Ele
  é instalado para todos os usuários, então o Windows pede permissão de administrador.
- **Permitir que o Windows PowerShell 5.1 rode scripts de perfil** (desligada por padrão): roda
  `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`. É uma **configuração de segurança**:
  deixa o Windows PowerShell 5.1 rodar scripts guardados neste computador, enquanto scripts
  baixados da internet continuam precisando de assinatura. Sem ela, o Windows PowerShell 5.1 não
  carrega o seu perfil, e o visual de terminal do ii só aparece no PowerShell 7, que já permite
  isso. Nada muda se a sua política já permite, e a desinstalação volta o valor anterior.

## O que o ii muda enquanto roda

- **Teclado e mouse:** o ii registra atalhos globais e instala um hook de baixo nível de teclado
  e um de mouse. Ele assume vários atalhos do Windows: a tecla Windows sozinha abre a busca do
  ii em vez do menu Iniciar, e Win+Tab, Win+V, Win+A, Win+N, Win+1...0, Win+Q (fecha a janela
  ativa) e Win+D (maximiza) fazem coisas do ii. Win+/ mostra todos; você pode mudá-los em
  `%LOCALAPPDATA%\illogical-impulse\keybinds.json`. Tudo isso para quando o ii fecha.
- **Barra de tarefas:** por padrão a barra de tarefas do Windows fica escondida e aparece quando
  o ponteiro encosta na borda de baixo da tela. Para isso, o ii liga a ocultação automática da
  barra enquanto roda (se ela estava desligada) e a desliga de novo quando fecha.
- **Papel de parede e cores:** na primeira vez que abre, o ii usa o papel de parede atual do
  Windows sem mudá-lo. Quando você escolhe um papel de parede ou alterna entre claro e escuro no
  ii, ele aplica o mesmo no Windows: o papel de parede da área de trabalho, o modo claro/escuro
  do Windows e a cor de destaque.
- **Windows Terminal:** o ii grava um esquema de cores igual ao tema dele, mais ajustes de fonte,
  cursor, margem e transparência para os perfis do PowerShell e do Prompt de Comando, como um
  *fragmento* do Windows Terminal em
  `%LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\illogical-impulse`. Ele não edita o
  `settings.json` do Windows Terminal; só atualiza a data desse arquivo para as janelas abertas
  recarregarem. Desligar o tema do terminal nas configurações do ii remove o fragmento.
- **Notificações:** o ii lê as notificações que o Windows mostra (pelo acesso a notificações que
  o Windows dá aos apps) para mostrá-las no painel dele. O Windows continua mostrando os próprios
  avisos também.
- **Áreas de trabalho virtuais:** os workspaces do ii são as áreas de trabalho virtuais do
  Windows. Ir para um workspace que ainda não existe cria uma nova área de trabalho.
- O que você muda pelos painéis do ii (volume e dispositivo de som, brilho, luz noturna, modo de
  energia, Bluetooth, Wi-Fi) são configurações comuns do Windows, mudadas pelas APIs do próprio
  Windows.

## Desinstalar

A desinstalação lê o `install-manifest.json` e desfaz o que está nele:

- fecha o ii; remove a entrada "Run", os atalhos do menu Iniciar e a entrada de Aplicativos
  instalados;
- tira o bloco marcado dos seus perfis do PowerShell (uma cópia de cada perfil de antes é salva
  em `%TEMP%`, e a desinstalação diz onde);
- volta a política de execução do PowerShell, se este instalador a mudou;
- desinstala com o winget as ferramentas de terminal que este instalador instalou (uma opção,
  ligada por padrão; o PowerShell 7 também, se foi este instalador que o instalou), e remove as
  fontes que ele instalou;
- apaga a pasta do fragmento do Windows Terminal, `%LOCALAPPDATA%\ii-windows`, as pastas do ii em
  `%LOCALAPPDATA%\quickshell` (`ii`, `State`, `cache`, `run`) e `%TEMP%\quickshell`;
- apaga as suas configurações em `%LOCALAPPDATA%\illogical-impulse`, a não ser que você marque
  *Manter minhas configurações*;
- volta o papel de parede, o modo claro/escuro, a cor de destaque e a ocultação automática da
  barra de tarefas anotados antes da primeira instalação, e garante que a barra de tarefas fique
  visível (uma opção, ligada por padrão).

Ela não consegue desfazer: áreas de trabalho virtuais que você ou o ii criaram; arquivos que
você fez (capturas de tela, gravações, papéis de parede baixados); mudanças que você fez pelo ii
em configurações comuns do Windows (volume, dispositivo de som, pareamentos Bluetooth, Wi-Fi,
modo de energia); um fundo em apresentação de slides ou Destaque do Windows (a imagem volta, a
apresentação não). Ferramentas e fontes que você já tinha antes de instalar continuam
instaladas.

## Reparar e atualizar

**Reparar** volta o ii para uma configuração básica de fábrica. Ele move as suas configurações
(`%LOCALAPPDATA%\illogical-impulse\*.json`) para uma pasta de backup com data,
`%LOCALAPPDATA%\illogical-impulse\backups\repair-<data>`, para o ii abrir com os padrões.
Depois reinstala os arquivos do programa da versão instalada (baixando-os se preciso), volta a
paleta de cores para a padrão, limpa o cache de QML do Quickshell, aplica de novo as opções com
que você instalou e abre o ii outra vez.

**Atualizar** procura uma versão mais nova no GitHub. Se houver, baixa e confere, substitui os
arquivos do programa e os arquivos do próprio ii, mantém as suas configurações e reabre o ii.

## Bom saber

- Este instalador e os programas do ii **não são assinados**. O SmartScreen do Windows pode dizer
  "O Windows protegeu o computador"; *Mais informações → Executar assim mesmo* abre. Faça isso só
  se você o baixou do link abaixo.
- É preciso o Windows 11 (build 22000 ou mais nova). O winget (Instalador de Aplicativo, que vem
  com o Windows 11) é necessário para as ferramentas de terminal.
- O log do instalador é `%LOCALAPPDATA%\ii-windows\setup.log`. Enquanto roda, o instalador
  guarda os arquivos temporários dele (o download, os dados de navegador da janela) em
  `%TEMP%\ii-windows-setup-...` e os apaga alguns segundos depois de fechar.

## Código-fonte e licenças

- ii-windows (empacotamento e este instalador):
  [github.com/nunreasonable/ii-windows](https://github.com/nunreasonable/ii-windows)
- Quickshell para Windows, branch `windows`:
  [github.com/nunreasonable/quickshell](https://github.com/nunreasonable/quickshell). O
  Quickshell é licenciado sob a LGPL-3.0.
- illogical-impulse para Windows, branch `ii-windows`:
  [github.com/nunreasonable/dots-hyprland](https://github.com/nunreasonable/dots-hyprland). O
  dots-hyprland do end-4 é licenciado sob a GPL-3.0.
- Também incluídos, entre outros: Qt 6 (LGPL-3.0), as bibliotecas FFmpeg do Qt Multimedia
  (LGPL-2.1 ou posterior), matugen (GPL-2.0), VirtualDesktopAccessor (MIT), as fontes
  JetBrainsMono Nerd Font, Rubik, Readex Pro, Space Grotesk e Google Sans Flex (SIL Open Font
  License), Material Symbols (Apache-2.0) e os ícones Adwaita (CC-BY-SA 3.0 / LGPL-3.0).
- Este instalador é feito com o Tauri (MIT / Apache-2.0) e mostra o texto na fonte Rubik (SIL
  Open Font License).
