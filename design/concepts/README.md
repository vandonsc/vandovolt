# Propostas visuais

Estas propostas foram criadas para orientar a escolha da direção visual do aplicativo.

**Direção selecionada:** A. Fluent Native. A implementação definitiva utiliza seus tokens de cor, hierarquia, superfícies e proporções como base.

## A. Fluent Native

- Integração visual com Windows 11 e seus tons neutros.
- Superfície discreta para o ícone, raio de 14 px e azul semântico moderado.
- Hierarquia convencional, familiar e de menor risco para temas e DPI variados.

## B. Premium Minimal

- Painel mais editorial, inspirado em Linear e Raycast.
- Mais espaço negativo, ícone sem superfície decorativa e acento violeta.
- Raio de 18 px e contraste mais marcado entre tipografia principal e metadados.

## C. Monochrome

- Quase totalmente monocromático, com foco em tipografia e forma.
- Menor raio, iconografia mais discreta e ausência de cores de marca.
- Estados continuam sendo comunicados por texto, forma, porcentagem e progresso, não apenas por cor.

## Elementos comuns

- Segoe UI, números em semibold/bold e textos secundários com contraste controlado.
- Escala de espaçamento baseada em 4 px.
- Iconografia com curvas contínuas, terminais arredondados e margens ópticas.
- Prévias de ícones a 16 px em uma barra simulada e ampliações 8× para inspeção.

As imagens foram renderizadas com GDI+ (`System.Drawing`) em modo antialias e ClearType. Elas são propostas de direção visual, não capturas do aplicativo. Após a escolha, o renderizador e as geometrias selecionadas deverão ser implementados no código nativo e validados no System Tray real.
