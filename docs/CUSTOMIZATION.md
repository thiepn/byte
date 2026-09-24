# Byte Customization & Decoration — Phase 13

Phase 13 turns the previously reserved Customize surface into a persistent, live companion personalization system.

## Persistent model

Configuration schema version 4 adds one customization object to CompanionPreferences.

Character customization stores one selection for each semantic anchor category:

- headwear
- face accessory
- body accessory
- back accessory
- hand prop

Habitat customization stores one selection for each of the six fixed Phase 11/12 slots:

- large_background
- wall_or_sky
- surface_left
- surface_right
- small_prop
- ambient

All selections default to `none`. Config versions 1–3 migrate to schema 4 without losing the existing character, palette, habitat, window mode, size, interaction level, edge anchor, or saved placements.

## Character cosmetics

Phase 13 ships twelve local pixel-style SVG cosmetics:

- Beanie
- Tiny Crown
- Sprout
- Round Glasses
- Star Glasses
- Cozy Scarf
- Bow Tie
- Backpack
- Little Wings
- Mug
- Book
- Star Wand

Cosmetics attach through the seven semantic anchors already authored into every production character frame. Hats therefore follow the head during sleep and movement, props follow the right hand during typing and reactions, and back items stay behind the character.

There are no character-specific renderer branches.

## Habitat decorations

The six fixed habitat slots now accept a small curated decoration catalog:

- Pennant Banner / Constellation
- String Lights / Paper Cloud
- Potted Plant / Book Stack
- Tiny Lamp / Tiny Radio
- Tiny Mug / Little Crystal
- Star Mobile / Sparkle Chimes

Decoration geometry is local to the slot and is placed using the selected habitat manifest's own x/y, plane, and order. Changing from Meadow to Space therefore keeps the same semantic selection but repositions it according to the new world.

Freeform dragging of furniture is intentionally out of scope.

## Live application

The full application Customize destination now provides:

- character selection
- all eight palettes for the active character
- habitat selection
- all five cosmetic categories
- all six decoration slots
- display mode selection
- companion size selection
- one-click accessory/decor reset

Saving a full companion preference update emits a local Tauri event directly to the companion window. The companion applies visual changes without restarting the app.

Character or habitat changes use revision guards so slower asset loads cannot overwrite a newer selection.

## Failure behavior

- unknown cosmetic ids render nothing
- missing cosmetic SVGs are skipped instead of crashing the companion
- decorations selected for the wrong slot are ignored
- invalid character/habitat ids retain the existing fallback behavior
- palette ids unsupported by a newly selected character fall back to that character's default palette

## Privacy and performance

All customization assets are bundled locally.

There is:

- no account
- no store
- no currency
- no unlock timer
- no cloud inventory
- no network asset fetch
- no separate cosmetic animation loop

The character and habitat still render on the shared 12 FPS companion scheduler.

Next: Phase 14 — Personality & Ambient Behavior.


## Phase 18 Studio layer

Phase 18 does not replace the Phase 13 persistence model. It turns that model into the intended full Customization Studio.

The Studio adds:

- a large live animated preview using the production character and habitat renderers
- character-focus / full-habitat preview switching
- manual Happy, Curious, Typing, and Sleep reaction previews
- visual category navigation rather than one long settings form
- curated built-in looks
- complete visual browsing for characters, palettes, cosmetics, habitats, decorations, personalities, display modes, and size
- instant optimistic changes with serialized persistence
- Undo and Redo for the current Studio session
- outfit, decor, and whole-look reset paths
- clear local save state

All Studio operations write the existing CompanionPreferences shape. There is no parallel draft file or second customization database.

See [CUSTOMIZATION_STUDIO.md](CUSTOMIZATION_STUDIO.md).
