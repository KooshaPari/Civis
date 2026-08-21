# i18n
> Internationalization and localization for the Civis platform.

## Overview

The `i18n` crate provides a complete internationalization (i18n) and localization (l10n) framework for the Civis simulation. It manages translation catalogs, plural rules, date/number formatting, and locale detection across the server and client tiers.

Translation data is stored in TOML-based resource bundles and compiled into a static lookup table at build time. This ensures zero-cost string resolution in hot paths while still allowing runtime bundle reloading for development and modding workflows.

The crate supports right-to-left (RTL) scripts, Unicode segmentation, and locale-aware collation. All public API surfaces return borrowed string slices to avoid unnecessary allocations in rendering loops.

## Features

- Compile-time translation bundle compilation
- Runtime hot-reload of translation bundles
- Plural rule engine (CLDR-compatible)
- Locale detection from system and user preferences
- Unicode segmentation and RTL support
- Locale-aware number and date formatting
- Missing-translation fallback chains

## Usage

```rust
use i18n::{Locale, Translator};

let translator = Translator::load("en-US", "./bundles")?;
let greeting = translator.translate("citizen.greeting", &["Alice"]);
println!("{}", greeting);
```

## Architecture

The crate is split into three layers: (1) a build-time compiler that turns TOML bundles into optimized Rust lookup tables, (2) a runtime `Translator` that resolves keys with locale fallback, and (3) formatting utilities that integrate with the `icu` crate family for CLDR compliance.

## License

Part of the Civis project (https://github.com/KooshaPari/Civis).
