# Antes de instalar

O **illogical-impulse** (ii) é o shell de desktop do end-4 para o Hyprland no Linux: barra,
painéis laterais, visão geral, lançador, notificações e cores Material You tiradas do papel de
parede. O **ii-windows** é um port não oficial dele para o Windows 10 e 11, rodando numa versão
do Quickshell para Windows. É experimental: espere arestas.

Esta página lista o que este instalador muda no seu computador e o que o ii muda enquanto
roda. Tudo é instalado só para o seu usuário do Windows; não é preciso permissão de
administrador, a não ser para o PowerShell 7 opcional.

## O que é instalado, e onde

- **O programa** vai para `%LOCALAPPDATA%\ii-windows` (cerca de 250 MB): o Quickshell
  (`qs.exe`, `qsw.exe`), as bibliotecas do Qt, as fontes e os ícones que o ii usa, o
  `matugen.exe` (gera a paleta de cores a partir do papel de parede) e a
  `VirtualDesktopAccessor.dll` (duas versões: uma para o Windows 11 e uma para o Windows 10, em
  `win10`) e as DLLs do runtime do Microsoft Visual C++ de que o programa precisa
  (`msvcp140*.dll`, `vcruntime140*.dll`), só nessa pasta: nada é instalado nas pastas do próprio
  Windows. Também o `songrec.exe` (reconhece músicas, do SongRec) e o `LaTeX.exe` com a pasta
  `res` (desenha fórmulas no chat de IA, do MicroTeX), com as licenças deles em `licenses`. Uma
  cópia deste instalador também fica lá, junto com o
  `install-manifest.json` (o registro de tudo que este instalador mudou, usado para desfazer) e o
  `setup.log`.
- **Os arquivos do próprio ii** vão para `%LOCALAPPDATA%\quickshell\ii`. Eles são substituídos a
  cada instalação, atualização e reparo, então não os edite.
- **As suas configurações** ficam em `%LOCALAPPDATA%\illogical-impulse` (`config.json` e
  outros). O ii cria esses arquivos conforme você muda as coisas; instalar e atualizar nunca
  mexem neles.
- **O estado e o cache do ii** (cores atuais, lista de tarefas, histórico de notificações, logs,
  relatórios de falha) vão para `%LOCALAPPDATA%\quickshell` (`State`, `cache`, `run`), as
  miniaturas dos papéis de parede e as imagens do histórico da área de transferência para
  `%LOCALAPPDATA%\cache` (`thumbnails`, `quickshell`), e os arquivos temporários para
  `%TEMP%\quickshell`. Uma paleta de cores padrão é colocada se ainda não houver nenhuma.
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
  - no Windows 10, se o winget (Instalador de Aplicativo) ainda não estiver lá, uma opção dentro
    desta (ligada por padrão) baixa e o instala primeiro: veja *winget* em "Windows 10" abaixo;
  - o *Oh My Posh*, o *Starship* e o *eza* são instalados com `winget install --scope user`, a
    não ser que já estejam lá; no Windows 10, o *Windows Terminal* é instalado do mesmo jeito se
    ele ainda não estiver lá (no Windows 11 ele já está, então nada acontece lá). O winget roda
    com `--accept-package-agreements` e `--accept-source-agreements`, ou seja, este instalador
    aceita por você as licenças desses pacotes e os termos da fonte do winget. Só as ferramentas
    que este instalador instalou são lembradas para a desinstalação;
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
- **Instalar o FFmpeg** (desligada por padrão): `winget install --scope user Gyan.FFmpeg`, a
  versão do `ffmpeg`, `ffplay` e `ffprobe` do [gyan.dev](https://www.gyan.dev/ffmpeg/). O ii vai
  ganhar o próprio gravador de tela nativo; esta opção é um gravador alternativo que ele pode usar
  no lugar, e o comando `ffmpeg` também é útil por conta própria. É uma opção separada da
  *Configuração do terminal* (não precisa dela ligada), mas ainda precisa do winget: sem ele, esta
  opção é pulada do mesmo jeito que as ferramentas de terminal, e no Windows 10 a mesma opção de
  instalar o winget primeiro (veja *winget* em "Windows 10" abaixo) o instala para esta opção
  também, se qualquer uma das duas precisar. O Gyan.FFmpeg é um pacote `zip`/portátil sem escopo
  de instalação declarado, então o winget o instala por usuário, do mesmo jeito que o Oh My Posh,
  o Starship e o eza acima (com os mesmos `--accept-package-agreements`/`--accept-source-agreements`).
  A desinstalação o remove só se foi este instalador que o instalou, assim como o PowerShell 7.
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
- **Bandeja do sistema:** a barra do ii mostra os ícones que os apps põem na área de
  notificação. Para isso, enquanto roda, o ii põe uma janela escondida na frente da bandeja do
  Explorer: os apps falam com ela, e ela repassa tudo ao Explorer, cuja bandeja continua
  funcionando. Quando o ii fecha ou trava, os apps voltam a falar direto com o Explorer. Enquanto
  o ii roda, ferramentas que ajustam a barra de tarefas procurando-a pela classe da janela podem
  não encontrá-la.
- **Fundo da área de trabalho:** o papel de parede do ii e os widgets dele (relógio, clima) ficam
  dentro da área de trabalho do Windows, atrás dos ícones, em vez de numa janela própria
  (Configurações > Fundo desliga isso). A pasta e os ícones da sua área de trabalho não mudam.
- **Papel de parede e cores:** na primeira vez que abre, o ii usa o papel de parede atual do
  Windows sem mudá-lo. Quando você escolhe um papel de parede ou alterna entre claro e escuro no
  ii, ele aplica o mesmo no Windows: o papel de parede da área de trabalho e o modo claro/escuro
  do Windows. A **cor de destaque** do Windows o ii muda sempre que carrega a paleta dele,
  inclusive toda vez que abre: ela passa a ser a cor principal da paleta do ii.
- **Windows Terminal:** o ii grava um esquema de cores igual ao tema dele, mais ajustes de fonte,
  cursor, margem e transparência para os perfis do PowerShell e do Prompt de Comando, como um
  *fragmento* do Windows Terminal em
  `%LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\illogical-impulse`. Ele não edita o
  `settings.json` do Windows Terminal; só atualiza a data desse arquivo para as janelas abertas
  recarregarem. Desligar o tema do terminal nas configurações do ii remove o fragmento.
- **Área de transferência:** o ii guarda um histórico do que você copia (até 200 itens), aberto
  com Win+V no lugar do histórico do Windows. O texto fica só na memória e some quando o ii fecha;
  as imagens copiadas são salvas em `%LOCALAPPDATA%\cache\quickshell\clipboard` e apagadas
  quando o ii fecha. O que os apps marcam para ficar fora do histórico (como senhas de
  gerenciadores de senha) não entra.
- **Gravação de tela:** o ii grava com a captura de tela e o codificador de vídeo do próprio
  Windows (Media Foundation), na sua pasta Vídeos ou no caminho que você escolher; nenhum programa
  extra é necessário. "Com som" grava o que sai nos alto-falantes, não o microfone. No Windows 10,
  o Windows desenha uma borda amarela em volta do que está sendo capturado.
- **Tradução:** o tradutor da barra lateral esquerda do ii (desligado até você ligar nas
  Configurações) e o tradutor de tela mandam o texto a traduzir para o serviço gratuito de tradução
  do Google (translate.googleapis.com). O tradutor de tela primeiro lê o texto da tela com o
  reconhecimento de texto do próprio Windows, no computador.
- **Reconhecimento de música:** quando você liga, o ii grava alguns segundos do que sai nos
  alto-falantes (ou do microfone, se você escolher) e manda uma impressão digital do áudio para os
  servidores do Shazam para identificar a música, como o SongRec faz no Linux.
- **Chaves de API:** se você salvar uma chave de API no painel de IA do ii, ela fica no
  Gerenciador de Credenciais do Windows, numa credencial chamada `illogical-impulse`.
- **Notificações:** o ii lê as notificações que o Windows mostra (pelo acesso a notificações que
  o Windows dá aos apps) para mostrá-las no painel dele. O Windows continua mostrando os próprios
  avisos também.
- **Áreas de trabalho virtuais:** os workspaces do ii são as áreas de trabalho virtuais do
  Windows. Ir para um workspace que ainda não existe cria uma nova área de trabalho.
- O que você muda pelos painéis do ii (volume e dispositivo de som, modo de energia, Bluetooth,
  Wi-Fi) são configurações comuns do Windows, mudadas pelas APIs do próprio Windows. O **brilho**
  é o da tela do notebook pelo Windows; num monitor externo, o ii muda o brilho do próprio
  monitor (por DDC/CI), e ele fica assim. Num monitor que não aceita DDC/CI (desligado no menu
  do monitor, ou a tela de uma máquina virtual), o ii escurece a tela pela gama dela, até mais ou
  menos a metade, e isso volta ao normal quando o ii fecha. A **luz noturna** do ii não é a do
  Windows: ela também ajusta a gama da tela e tudo volta ao normal quando o ii fecha.

## Desinstalar

A desinstalação lê o `install-manifest.json` e desfaz o que está nele:

- fecha o ii; remove a entrada "Run", os atalhos do menu Iniciar e a entrada de Aplicativos
  instalados;
- tira o bloco marcado dos seus perfis do PowerShell (uma cópia de cada perfil de antes é salva
  em `%TEMP%`, e a desinstalação diz onde);
- volta a política de execução do PowerShell, se este instalador a mudou;
- desinstala com o winget as ferramentas de terminal que este instalador instalou (uma opção,
  ligada por padrão; o PowerShell 7 e o FFmpeg também, cada um como sua própria opção, se foi
  este instalador que os instalou), depois remove o próprio winget (`Remove-AppxPackage`) se foi
  este instalador que o instalou no Windows 10, e remove as fontes que ele instalou;
- apaga a pasta do fragmento do Windows Terminal, `%LOCALAPPDATA%\ii-windows`, as pastas do ii em
  `%LOCALAPPDATA%\quickshell` (`ii`, `State`, `cache`, `run`), `%LOCALAPPDATA%\cache\quickshell`
  e `%TEMP%\quickshell`, e também `%LOCALAPPDATA%\cache\thumbnails` se essa pasta não existia
  antes da instalação;
- apaga as suas configurações em `%LOCALAPPDATA%\illogical-impulse` e a credencial
  `illogical-impulse` (as chaves de API), a não ser que você marque *Manter minhas
  configurações*;
- volta o papel de parede, o modo claro/escuro, a cor de destaque e a ocultação automática da
  barra de tarefas anotados antes da primeira instalação, e garante que a barra de tarefas fique
  visível (uma opção, ligada por padrão).

Ela não consegue desfazer: áreas de trabalho virtuais que você ou o ii criaram; arquivos que
você fez (capturas de tela, gravações, papéis de parede baixados); mudanças que você fez pelo ii
em configurações comuns do Windows (volume, dispositivo de som, pareamentos Bluetooth, Wi-Fi,
modo de energia) e no brilho de monitores externos; um fundo em apresentação de slides ou Destaque do Windows (a imagem volta, a
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
- É preciso o Windows 10 versão 2004 ou mais nova (build 19041; 22H2, build 19045, recomendada)
  ou o Windows 11 (build 22000 ou mais nova). O winget (Instalador de Aplicativo) é necessário
  para as ferramentas de terminal e para o FFmpeg.
- Toda vez que abre, o instalador procura o último release no GitHub. Se ele mesmo for mais
  velho, baixa o instalador mais novo desse release, confere com o SHA-256 que o GitHub publica
  para ele e reabre com ele (não quando aberto com `--package`). Um pacote ao lado do instalador
  mais velho que o último release é deixado de lado pelo mais novo. Sem internet, ele segue com o
  que tem.
- O log do instalador é `%LOCALAPPDATA%\ii-windows\setup.log`. Enquanto roda, o instalador
  guarda os arquivos temporários dele (o download, os dados de navegador da janela) em
  `%TEMP%\ii-windows-setup-...` e os apaga alguns segundos depois de fechar.

## Windows 10

O ii-windows também instala no Windows 10 versão 2004 ou mais nova (build 19041; 22H2, build
19045, recomendada), ao lado do Windows 11. Algumas coisas funcionam diferente lá:

- Abaixo da build 19041, o instalador se recusa a instalar: uma verificação na página de opções
  bloqueia, com uma mensagem dizendo que é preciso o Windows 10 versão 2004 ou mais nova, 22H2
  recomendada, ou o Windows 11. Da 19041 para cima, a verificação passa e identifica o sistema
  como "Windows 10" com a versão e a build dele; da build 22000 para cima ele é identificado como
  "Windows 11", como antes.
- O instalador precisa do **Microsoft Edge WebView2 Runtime** para conseguir mostrar a janela
  dele. O Windows 11 sempre tem; se estiver faltando no Windows 10, o instalador mostra uma
  caixa de mensagem, antes de qualquer outra coisa, oferecendo abrir a página de download do
  WebView2 da Microsoft, e depois fecha.
- O **winget** (Instalador de Aplicativo) também não vem com o Windows 10; ele costuma chegar
  depois, pela Microsoft Store. Se a *Configuração do terminal* ou o *Instalar o FFmpeg*
  estiverem ligados e o winget não for encontrado, este instalador pode instalá-lo primeiro,
  antes das ferramentas de terminal e do FFmpeg abaixo: uma opção mostrada ao lado dessas, ligada
  por padrão, baixa o pacote do Instalador de Aplicativo e as dependências dele (cerca de 300 MB
  juntos) da versão mais nova do [microsoft/winget-cli](https://github.com/microsoft/winget-cli)
  no GitHub, e o instala para o seu usuário com `Add-AppxPackage` - sem precisar de permissão de
  administrador. Algumas versões do winget-cli publicam o SHA-256 do msixbundle ao lado dele;
  quando isso existe, é conferido antes de instalar, e este instalador avisa; quando não existe,
  este instalador não inventa uma verificação própria, já que o Windows confere a assinatura da
  Microsoft do pacote ao instalar, de qualquer jeito. Desligar essa opção pula essa etapa (como
  antes: as ferramentas de terminal e o FFmpeg também são pulados). A desinstalação o remove com
  `Remove-AppxPackage`, só se foi este instalador que o instalou. Um download ou instalação que
  falhe aqui é um aviso, não um instalador que falhou: as ferramentas de terminal e o FFmpeg são
  então pulados, exatamente como quando o winget nunca foi encontrado.
- O **Windows Terminal** não vem com o Windows 10. Se a opção *Configuração do terminal* estiver
  ligada e o `wt.exe` não for encontrado (e o winget também não listar ele como instalado), este
  instalador o instala com `winget install --scope user Microsoft.WindowsTerminal`, do mesmo
  jeito que o Oh My Posh, o Starship e o eza; a desinstalação só o remove se foi este instalador
  que o instalou. No Windows 11, onde ele normalmente já está lá, isso não faz nada.

O próprio ii também muda um pouco no Windows 10:

- **Barra de tarefas:** no Windows 10 a barra de tarefas pode ficar em qualquer borda da tela. A
  barra que só aparece com o ponteiro acompanha: ela aparece quando o ponteiro encosta na borda
  em que ela está.
- **Áreas de trabalho virtuais:** o Windows 10 não dá ao ii um jeito direto de criar ou remover
  áreas de trabalho. Ir para um workspace que ainda não existe cria a área de trabalho
  apertando *Ctrl+Win+D* por você, como você faria à mão, e o ii não consegue remover áreas de
  trabalho lá.
- **O desfoque atrás dos painéis** usa o efeito de desfoque mais antigo do Windows 10, com os
  cantos arredondados um pouco serrilhados.
- **Terminal:** os atalhos de terminal abrem o Windows Terminal, ou o Windows PowerShell onde o
  Windows Terminal não está instalado. Sem o winget (Instalador de Aplicativo, que nem todo
  Windows 10 tem), as ferramentas de terminal, o Windows Terminal, o PowerShell 7 e o FFmpeg não
  podem ser instalados e são pulados.

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
  (LGPL-2.1 ou posterior), matugen (GPL-2.0), VirtualDesktopAccessor (MIT), o runtime do
  Microsoft Visual C++ (arquivos redistribuíveis, sob os termos de licença da Microsoft), as fontes
  JetBrainsMono Nerd Font, Rubik, Readex Pro, Space Grotesk e Google Sans Flex (SIL Open Font
  License), Material Symbols (Apache-2.0) e os ícones Adwaita (CC-BY-SA 3.0 / LGPL-3.0).
- `songrec.exe`: o reconhecedor do SongRec (GPL-3.0 ou posterior). O código-fonte é o SongRec
  0.7.5 ([github.com/marin-m/SongRec](https://github.com/marin-m/SongRec)) mais a interface de
  linha de comando para Windows em `tools/songrec` deste projeto.
- `LaTeX.exe`: MicroTeX (MIT), com o tinyxml2 (zlib) e as fontes em `res` sob as licenças delas
  (algumas GPL-3.0), todas em `licenses\microtex`.
- Este instalador é feito com o Tauri (MIT / Apache-2.0) e mostra o texto na fonte Rubik (SIL
  Open Font License).
