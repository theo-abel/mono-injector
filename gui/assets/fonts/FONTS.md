# Embedded Fonts

The GUI embeds these fonts at compile time via `include_bytes!` in
`gui/src/main.rs`. A clean checkout must include the font files listed here or
the GUI crate will fail to compile.

## Font Files

- `HankenGrotesk-Regular.ttf`
- `HankenGrotesk-SemiBold.ttf`
- `JetBrainsMono-Regular.ttf`
- `JetBrainsMono-Medium.ttf`
- `JetBrainsMono-Bold.ttf`
- `MaterialSymbolsOutlined.ttf`

## Sources And Licenses

- Hanken Grotesk
- Source: https://fonts.google.com/specimen/Hanken+Grotesk and https://github.com/marcologous/hanken-grotesk
- License: SIL Open Font License 1.1
- License file: `LICENSE-HankenGrotesk.txt`

- JetBrains Mono
- Source: https://www.jetbrains.com/lp/mono/ and https://github.com/JetBrains/JetBrainsMono
- License: SIL Open Font License 1.1
- License file: `LICENSE-JetBrainsMono.txt`

- Material Symbols Outlined
- Source: https://fonts.google.com/icons and https://github.com/google/material-design-icons
- License: Apache License 2.0
- License file: `LICENSE-MaterialSymbols.txt`

Do not rename or modify the OFL-licensed font binaries unless the OFL reserved
font name requirements are checked first.
