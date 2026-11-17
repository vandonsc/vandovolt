# Peripheral Battery 0.1.0

Aplicativo Windows 11 x64 em Rust, Win32/GDI e `hidapi`. Exibe as baterias do HyperX Cloud III Wireless e do NZXT Lift Elite Wireless como dois ícones independentes na área de notificação do Windows.

## Compilação e instalação

1. Instale o Rust com o toolchain MSVC, o Visual Studio Build Tools e o Windows SDK.
2. Execute `compilar.cmd`. A primeira compilação pode precisar baixar as dependências do Cargo.
3. Feche NGENUITY, NZXT CAM e outros aplicativos que possam consultar os mesmos dispositivos HID.
4. Execute `target\x86_64-pc-windows-msvc\release\peripheral-battery.exe`.
5. Para instalar em `%LOCALAPPDATA%\PeripheralBattery` e iniciar com o Windows, execute `instalar.cmd`.

`desinstalar.cmd` remove o executável instalado, a configuração local e a entrada de inicialização automática.

## Uso

- Cada periférico possui seu próprio ícone e tooltip com nome, nível e estado de carregamento.
- Clique com o botão esquerdo em qualquer ícone para atualizar imediatamente.
- Clique com o botão direito para abrir o menu de configurações.
- Estilos disponíveis: ícone com porcentagem compacta, somente porcentagem e ícone com indicador visual de bateria.
- Os ícones são renderizados em resolução 4× e reduzidos com transparência alpha para suavizar curvas, texto e contornos.
- A geometria visual segue proporções inspiradas em ícones Fluent/Lucide, com traços de aproximadamente 2 px no tamanho final e ocupação de até 94% da área útil.
- O estilo “Ícone + porcentagem” utiliza composição vertical: símbolo na parte superior e número em semibold na parte inferior, evitando a compressão horizontal.
- No estilo de indicador proporcional, a carga é preenchida dentro do mouse ou das conchas do headset.
- Nesse estilo, o contorno permanece neutro e independente do preenchimento: off-white para dispositivos conectados e cinza para indisponíveis. A carga usa uma área interna recuada, evitando que níveis altos ocultem a silhueta.
- Intervalos disponíveis: 1, 5, 10, 15 ou 30 minutos. O padrão é 5 minutos.
- Por padrão, manter o ponteiro sobre um ícone abre após aproximadamente 500 ms um painel compacto com dispositivo, conexão, carga, barra de progresso, estado e tempo desde a última atualização. A opção pode ser desativada pelo menu.
- O painel segue a direção “Fluent Native”, acompanha `AppsUseLightTheme` e usa superfície neutra para o periférico, status com indicador visual e texto, porcentagem em destaque e progresso arredondado.
- Escala tipográfica do painel: dispositivo em 15,5 px semibold, status em 12,5 px, bateria em 32 px bold e última atualização em 12 px.
- As preferências são gravadas em `%LOCALAPPDATA%\PeripheralBattery\config.ini`.
- Branco indica estado normal, vermelho indica 10% ou menos, verde indica carregamento e cinza indica dispositivo indisponível.
- Os alertas de bateria baixa são emitidos uma vez por ciclo, até o nível subir acima de 10% ou o dispositivo entrar em carregamento.

O aplicativo mantém apenas uma janela Win32 invisível para receber mensagens. Os ícones são recriados após a mensagem `TaskbarCreated`, enviada quando o Explorer restaura a barra de tarefas.

## Desempenho e limitações

- Não há polling de interface, animação, navegador, rede, serviço ou injeção no Explorer.
- A consulta HID ocorre em uma thread curta no intervalo configurado; os ícones só são regenerados quando estado, estilo, DPI ou configurações visuais mudam.
- O painel usa uma janela Win32 temporária sem ativação, fora do Alt+Tab, posicionada a partir de `Shell_NotifyIconGetRect`. Nenhuma consulta HID é realizada ao exibi-lo.
- A apresentação do painel é renderizada com GDI+ em modo antialias, incluindo curvas, superfícies, progresso e tipografia Segoe UI Variable/Segoe UI.
- Os símbolos ampliados do painel são vetores GDI+ próprios e não reutilizam o `HICON` compacto, evitando fundos opacos e permitindo proporções mais refinadas.
- A visibilidade e a ordem dos ícones são controladas pelo próprio Windows. O aplicativo não altera configurações não documentadas para forçá-las.
- Por causa do espaço reduzido dos ícones da área de notificação, o estilo “Ícone + porcentagem” usa representação compacta. O tooltip sempre apresenta o valor completo.
- Aparência, DPI por monitor e consumo de recursos devem ser confirmados visualmente no sistema de destino; a compilação automatizada não substitui essa validação.
- Caso o Windows não forneça a posição do ícone, o aplicativo mantém o tooltip nativo como fallback em interações posteriores.

## Prévias de design

As prévias em escala 2× ficam em `design/previews/`, cobrindo temas claro e escuro, headset e mouse, bateria baixa, estado normal e carregamento. Elas podem ser regeneradas com `design/generate_previews.ps1` e servem para revisão visual; não são capturas de tela do System Tray.

As matrizes `tray-mouse-states.png` e `tray-headset-states.png` mostram o indicador integrado em 10%, 50%, 90% e nos estados de carregamento de 25%, 50%, 84% e 100%. Elas podem ser regeneradas com `design/generate_tray_state_previews.ps1`.

## Protocolos

A leitura HID permanece isolada em `src/devices.rs`. Ela utiliza apenas consultas de telemetria e não altera firmware ou configurações dos periféricos.
