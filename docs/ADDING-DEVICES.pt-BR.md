# Adicionando um dispositivo

O HIDra começou com dois periféricos Redragon porque eram os que estavam na mesa
do autor — nada nele é preso a essa marca. A tabela de dispositivos é uma lista
simples de IDs USB e nomes de protocolo, então um mouse Logitech, Razer,
Corsair, Motospeed, Havit, Fantech ou sem marca cabe ali exatamente como um
Redragon. O objetivo é uma biblioteca de compatibilidade que só cresce.

São dois casos:

1. **Seu aparelho fala um protocolo que o HIDra já implementa.** É o mais comum:
   periféricos costumam ser a mesma placa OEM vendida com marcas diferentes,
   mudando apenas o product id. É uma linha, e não precisa saber Rust.
2. **Seu aparelho fala algo novo.** Aí é preciso capturar o protocolo, e há uma
   receita no fim desta página.

---

## Versão curta

```sh
git clone https://github.com/luizjr/HIDra && cd HIDra
python3 scripts/probe-device.py          # descobre os números e imprime a entrada
# cole a entrada em src-tauri/src/devices.rs e escolha o protocolo
python3 scripts/sync-devices.py          # atualiza regra udev, READMEs e site
npm install && npm run tauri dev         # leia um ajuste, mude um ajuste
```

Depois abra um pull request dizendo o que você testou e em qual sistema. É esse
o fluxo inteiro; o resto da página explica cada passo.

---

## 1. Identificar o aparelho

A configuração não trafega pela coleção que digita letras ou move o ponteiro —
ela trafega por uma **coleção definida pelo fabricante** (usage page HID na
faixa `0xFF00`–`0xFFFF`), quase sempre na interface USB 1.

No Linux, o script de sondagem acha isso para você:

```sh
python3 scripts/probe-device.py
```

Ele lista cada interface USB HID que tenha coleção de fabricante, os relatórios
de cada uma e o palpite de onde se escreve e onde o aparelho responde:

```
Compx · 2.4G Wireless Receiver
  25a7:fa07  interface 1  /dev/hidraw2
    usage page 0xff01 usage 0x00  0x09 I16B  <-- replies arrive here
    usage page 0xff02 usage 0x02  0x08 F16B  <-- config: write here
  → config_usage_page: 0xff02   reply_usage_page: 0xff01
```

O palpite vem do tamanho dos relatórios: o canal de configuração costuma ser o
maior relatório de saída (Output) ou de feature da interface. É um palpite — se
as escritas não surtirem efeito depois, volte e tente outra coleção.

`--issue` imprime o mesmo texto pronto para colar numa issue, e `--json` existe
para scripts e agentes de IA.

**No Windows**: Gerenciador de Dispositivos → o aparelho → Detalhes → *IDs de
hardware* dá `HID\VID_25A7&PID_FA08`. Para ver as coleções, use o USBView ou o
Wireshark com USBPcap.

**No macOS**: Informações do Sistema → USB dá os IDs; o descritor de relatório
exige uma ferramenta como o `hidapitester` ou `ioreg -l -w0 | grep -i HIDReport`.

Os dois dão mais trabalho que o script no Linux — se você tiver uma máquina
Linux por perto, use-a nesta etapa mesmo que use outro sistema no dia a dia.

## 2. Escolher o protocolo

O enum `Protocol`, em `src-tauri/src/devices.rs`, lista o que o HIDra já fala:

| Protocolo | Formato | Conhecido em |
| --- | --- | --- |
| `Compx17` | relatórios de feature de 17 bytes no id `0x08`, cada pacote somando `0x55` | mouses Compx/Areson (família M711-PRO) |
| `Sonix64` | relatórios de 64 bytes no id `0x04`, checksum de soma 16 bits | teclados SONiX SN32F (família K586RGB-PRO) |

Sinais de que o seu aparelho fala um deles:

- o programa oficial dele se parece com o de um aparelho que já suportamos
  (mesmo layout, mesmos nomes de efeito, mesma quantidade de perfis);
- os tamanhos de relatório batem (`16B` de feature aponta para `Compx17`;
  relatórios de ~63B numa única coleção de fabricante, para `Sonix64`);
- o controlador é o mesmo — uma foto do desmonte ou um registro da FCC citando
  um SONiX SN32F ou um chip Compx é evidência forte.

Na dúvida, o jeito honesto de descobrir é testar. Marcar
`support: Support::Reported` deixa a entrada como não comprovada, e o HIDra
mostra um selo de aviso ao lado do aparelho — assim um palpite errado fica
visível em vez de enganar em silêncio.

## 3. Adicionar a entrada

Cole o que o script imprimiu dentro de `SUPPORTED`, em
[`src-tauri/src/devices.rs`](../src-tauri/src/devices.rs), e preencha o que ele
não tinha como saber:

```rust
Device {
    vendor: "Redragon",                  // marca na caixa
    model: "Cobra Pro (M711-PRO)",       // nome comercial + código do modelo
    vid: 0x25a7,
    pid: 0xfa08,
    kind: Kind::Mouse,                   // define qual painel ele recebe
    protocol: Protocol::Compx17,
    interface: 1,                        // interface USB do canal de config
    config_usage_page: 0xff02,           // coleção onde se escreve
    reply_usage_page: 0xff01,            // coleção onde ele responde
    connection: Connection::Wired,       // ou Receiver, para dongle 2.4 GHz
    support: Support::Reported,          // Verified quando funcionar na sua mesa
},
```

Um aparelho que aparece com product ids diferentes no cabo e no receptor recebe
**uma entrada para cada id** — é assim que o HIDra manda a escrita para as duas
interfaces e a que estiver acordada aceita.

`config_usage_page` e `reply_usage_page` são iguais na maioria dos aparelhos.
Eles diferem quando o aparelho responde numa coleção diferente daquela em que é
escrito (os mouses Compx fazem isso), e a distinção só importa no Windows, onde
cada coleção de topo é um handle separado. Errar isso lá significa escrever na
coleção de movimento do mouse e as escritas serem descartadas em silêncio.

## 4. Regerar o que depende da tabela

```sh
python3 scripts/sync-devices.py
```

Isso reescreve a regra udev, as tabelas dos dois READMEs e a tabela do site a
partir da sua entrada. O CI roda `--check` e falha se você esquecer, então não
tem como ficar desatualizado.

## 5. Testar no seu hardware

```sh
sudo sh packaging/install-udev.sh   # Linux, uma vez, e reconecte o aparelho
npm install
npm run tauri dev
```

Nesta ordem:

1. A barra lateral deve mostrar o aparelho conectado, com o nome do seu modelo.
2. **Leia** algo primeiro — o painel de iluminação ou DPI preenchido com os
   valores reais do aparelho prova que o endpoint está certo.
3. **Escreva** algo pequeno — brilho, ou um estágio de DPI — e veja o hardware
   reagir.
4. Leia de novo e confirme que a mudança ficou.

Se as leituras vierem vazias, provavelmente o endpoint está errado: tente outra
coleção de fabricante da saída do script. Se as leituras funcionam mas as
escritas não fazem nada num mouse sem fio, mexa no mouse enquanto aplica — o
rádio 2.4 GHz dorme quando parado e ignora configuração dormindo.

Quando tudo funcionar, troque `support` para `Support::Verified` e rode
`python3 scripts/sync-devices.py` de novo.

## 6. Enviar o pull request

```sh
npm run lint && npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
python3 scripts/sync-devices.py --check
```

Na descrição, diga **o que você testou, em qual sistema e o que viu**. Para um
aparelho novo, isso é a revisão inteira: ninguém mais tem o seu hardware, então
o seu relato é a evidência. "Coloquei brilho 3 no Fedora 41, li de volta 3, o
teclado escureceu" vale mais que qualquer revisão de código.

Um pull request que adiciona a entrada como `Support::Reported` sem teste também
é bem-vindo — diga isso claramente, e ele entra como não comprovado em vez de
ser recusado.

---

## Adicionando um protocolo novo

Quando nada em `Protocol` serve, é preciso capturar os pacotes do aparelho.

### Capturar

A receita que produziu o protocolo do teclado deste repositório, no Linux e sem
uma segunda máquina:

1. Rode o programa oficial (Windows) sob o Wine.
2. O `winebus` do Wine se recusa a entregar um nó `hidraw` de teclado ou mouse
   para o aplicativo, então espelhe o aparelho real num nó `/dev/uhid` e deixe o
   programa falar com o espelho. O proxy registra cada pacote nos dois sentidos.
3. Dirija a interface do programa com `xdotool` num `Xvfb` sem tela, mudando
   **um ajuste por vez**, para cada pacote capturado ter significado conhecido.

Com uma máquina Windows sobrando, USBPcap + Wireshark é mais simples. No Linux,
o `usbmon` (`modprobe usbmon`, e então capture `usbmon` no Wireshark) registra
tudo o que o kernel vê, inclusive o tráfego de um programa rodando numa VM com o
aparelho repassado.

### Implementar

- Acrescente uma variante em `Protocol`, em `devices.rs`, com um comentário
  dizendo como são os pacotes e qual módulo a implementa.
- Escreva o módulo ao lado de `mouse.rs` e `keyboard.rs`, seguindo o formato
  deles: montar pacote → checksum → aplicar → ler de volta.
- Abra o aparelho com `hid::open_speaking(api, kind, Protocol::SeuProtocolo)`,
  para que um aparelho de outro dialeto nunca receba os seus pacotes.
- Acrescente os textos de recursos do novo protocolo em `FEATURES`, dentro de
  `scripts/sync-devices.py`, para as tabelas geradas descreverem ele.
- Anote cada byte que você descobriu no [`PROTOCOL.md`](../PROTOCOL.md). As
  anotações valem tanto quanto o código: são elas que permitem à próxima pessoa
  consertar isso sem remontar a sua bancada de captura.

### O que o HIDra não faz

Apenas registradores de configuração — os mesmos que o programa oficial escreve.
Nunca comandos de bootloader ou gravação de firmware, nada que possa inutilizar
um aparelho. Um pull request que grava firmware será recusado, por melhor que
funcione.

---

## Usando um assistente de IA

Se você trabalha com o Claude Code ou um agente parecido, este repositório traz
instruções escritas para ele: o [`CLAUDE.md`](../CLAUDE.md) na raiz descreve a
arquitetura e as convenções, e `/add-device` é um comando pronto que percorre
todo o fluxo acima — sondar, entrada, regerar, compilar, plano de teste, pull
request. Aponte o agente para o repositório e peça para adicionar o seu
aparelho; ele tem tudo de que precisa. Você ainda precisa rodar o app e
confirmar que o hardware reagiu, porque nenhum agente enxerga o seu teclado
acender.
