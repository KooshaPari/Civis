# Keycap Palette fonts (Bevy egui HUD)

Bundled for offline HUD rendering (Phenotype Keycap Palette):

| File | Role |
|------|------|
| `Montserrat-Regular.ttf` | Body / UI (`FontFamily::Proportional`) |
| `Montserrat-SemiBold.ttf` | Strong labels (fallback chain) |
| `JetBrainsMono-Regular.ttf` | Numeric / coords (`FontFamily::Monospace`) |
| `BricolageGrotesque-SemiBold.ttf` | Display / headings (`FontFamily::Name("bricolage")`) |

Loaded once by `ui_theme::install_keycap_fonts`. If files are missing, egui defaults are used and a TODO is left in code.

Sources: [Montserrat](https://github.com/JulietaUla/Montserrat) (OFL), [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) (OFL), [Bricolage Grotesque](https://fonts.google.com/specimen/Bricolage+Grotesque) (OFL).
