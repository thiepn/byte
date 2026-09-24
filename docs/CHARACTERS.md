# Byte Production Characters

Phase 9 replaces the development sprite placeholder with four complete production character families.

All four characters use the same Phase 8 runtime contract, 64×64 animation canvas, semantic behavior vocabulary, and cosmetic-anchor system. Their art and motion language are intentionally different so changing characters feels meaningful rather than like recoloring the same mascot.

## Shared production standard

Each character ships with:

- 26 authored sprite poses
- complete coverage of every core semantic behavior
- idle / blink / look / sleep / wake
- left/right movement
- left/right typing pose
- fast typing loop
- click, pet, happy, surprised, annoyed
- busy, stressed, heat, memory pressure
- low battery, charging, network activity
- needs-attention behavior
- two character-specific rare idle poses
- reduced-motion fallback for every clip
- seven cosmetic attachment anchors on every frame
- eight hand-designed palettes
- 128px preview artwork for later customization UI

The atlas is 512×256 with 64×64 cells and nearest-neighbor rendering.

## Byte

Byte is the primary mascot: a compact rounded robot with warm amber antenna tips and a cool blue body.

Visual language:

- rectangular silhouette with soft corners
- digital visor expressions
- antenna sparkle as a rare idle
- tiny mug interaction as a rare idle
- tiny console/key lights while busy
- electrical bolt while charging
- heat marks when hot
- visible stack blocks during memory pressure

Palettes:

- Sky
- Mint
- Peach
- Lavender
- Cream
- Charcoal
- Rose
- Retro

## Mochi

Mochi is a round cat-like companion designed around immediate Bongo-Cat-style readability.

Visual language:

- triangular ears
- round body and visible tail
- oversized front paws
- alternating left/right paw taps
- drooped expression at low battery
- raised paws when happy/charging
- yarn-ball rare idle
- grooming-paw rare idle

Palettes:

- Cream
- Ginger
- Tuxedo
- Gray
- Lavender
- Strawberry
- Cocoa
- Snow

## Pip

Pip is the simplest silhouette and the most squash/stretch-oriented character.

Visual language:

- soft circular blob
- body compression for typing
- taller happy pose
- wide low-energy pose
- tiny split-off blob as a rare idle
- tiny crown/wobble rare idle
- clean facial expressions that remain readable at Mini size

Palettes:

- Sky
- Mint
- Peach
- Grape
- Lemon
- Rose
- Aqua
- Midnight

## Kiwi

Kiwi is the energetic bird companion.

Visual language:

- round body
- small legs
- large readable beak
- pecking left/right typing poses
- wing-up happy/charging pose
- seed interaction as a rare idle
- loose feather rare idle
- packet-like network particles

Palettes:

- Leaf
- Lime
- Autumn
- Bluebird
- Lavender
- Peach
- Snow
- Midnight

## Palette architecture

Palettes are semantic recolor maps rather than duplicated sprite sheets.

Every character defines exact production color slots:

- dark
- shadow
- primary
- highlight
- accent
- eye
- blush

The renderer recolors the loaded atlas once when a palette is selected, caches the recolored canvas as the active atlas, and then renders normal animation frames from it.

This keeps eight palettes per character lightweight and avoids loading 32 separate full sprite sheets.

The persisted palette ID is character-agnostic: default means use that character's own first/default palette. This allows existing configs to migrate safely even when the selected character is not Byte.

## Small-size readability

All character art is authored around a roughly 48px native silhouette inside the shared 64×64 frame.

That ensures:

- Mini mode does not shrink detailed 256px artwork into mush
- Perch mode keeps typing poses visible
- eyes remain at least one native pixel
- silhouette differences survive reduced display size
- cosmetic anchors remain within the visible canvas

The companion scene scales using nearest-neighbor canvas rendering and constrains character size by both window width and height.

## Cosmetic anchors

Every pose includes:

- head
- face
- body
- back
- left_hand
- right_hand
- ground

The anchors move with pose changes such as sleep, bounce, typing, and lateral movement.

Phase 13 can therefore attach hats, glasses, scarves, props, and back items without character-specific renderer branches.

## Validation

Production asset tests load all four manifests from public assets and verify:

- production status
- complete core behavior mappings
- at least 26 frames
- exactly eight palettes
- default palette availability
- every required anchor on every frame
- anchor bounds
- valid PNG atlas signature
- nontrivial atlas file size
- distinct base primary colors

Visual QA also checks each sprite sheet at its native scale. Automated validation cannot judge cuteness, so the later visual audit remains responsible for subjective animation quality, clipping, and polish.
