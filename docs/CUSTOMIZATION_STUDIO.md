# Byte Customization Studio — Phase 18

Phase 18 turns the existing customization controls into the intended **Customization Studio**.

The acceptance criterion is qualitative as well as functional: customizing Byte should feel like playing with the companion, not filling out a settings form.

## Large live preview

The Studio has a dedicated large preview column.

It renders the real selected:

- character
- palette
- cosmetics
- habitat
- fixed-slot decorations
- personality ambience
- local time-of-day habitat state

The preview uses the same production renderer classes as the desktop companion.

It does not maintain a separate sprite implementation.

### Preview modes

The user can switch between:

- full habitat
- character focus

The preview remains large even when the actual desktop companion is configured for Mini, Edge, Perch, or Tray. Display-mode controls affect the real companion immediately; they do not make the editing canvas unusably small.

### Reaction preview

The Studio can deliberately preview:

- Happy
- Curious
- Typing
- Sleep

These are temporary animation requests only. They are not saved as companion state.

Reduced-motion users do not receive the reaction-test strip.

## Studio navigation

Customization is split into six visual sections:

1. Looks
2. Character
3. Outfit
4. Habitat
5. Personality
6. Display

This replaces the Phase 13 single long vertical form.

## Looks

Phase 18 adds four built-in curated starting points:

- Cozy Reader
- Stargazer
- Garden Club
- Soft Night

A look is only a shortcut that writes ordinary CompanionPreferences. It is not a new persistent object.

Applying a look intentionally preserves shell-specific state such as saved window placements, current desktop size, and current display mode unless that control is changed separately.

## Character

The Character section presents all four production companions visually.

Changing character selects that character's authored default palette.

The Palette section then exposes all eight colorways for the selected character using actual palette colors rather than a text-only selector.

## Outfit

Outfit exposes the five existing semantic attachment categories:

- headwear
- face
- body
- back
- hand prop

Every item is shown visually.

The current Phase 18 catalog is anchor-compatible with all four production characters. There is still exactly one active item per category.

Clear outfit removes all five at once.

## Habitat

All six production habitats are available as visual choices.

The six fixed semantic decoration slots remain:

- large background
- wall / sky
- left surface
- right surface
- small prop
- ambient

The Studio does **not** introduce freeform furniture positioning. Clear decor returns all six slots to None.

## Personality

Chill, Curious, and Energetic are presented as visual personality cards with their actual behavioral descriptions.

Quiet / Normal / Playful remains a separate interaction-intensity control.

Personality continues to affect only low-priority expression and ambience. It cannot weaken diagnostics.

## Display

The Studio exposes:

- Habitat
- Perch
- Mini
- Edge
- Tray only
- Small / Medium / Large size

These controls persist through the same preference write used by every other Studio selection, so the real desktop companion updates immediately.

## Instant persistence

There is no Apply button.

When a selection changes:

1. the Studio preview updates optimistically
2. the latest full CompanionPreferences is queued
3. writes are serialized
4. the companion receives the existing preference-change event
5. the Studio shows Saving locally… or Saved locally

If another choice is made while a write is in flight, the newer full preference state waits behind it. This prevents an older asynchronous full-preference write from overwriting a newer choice.

A failed final write restores the last successfully saved preference state.

## Undo / Redo

The Studio keeps up to 20 local editing states for the current mounted Studio session.

Undo and Redo both write the resulting ordinary CompanionPreferences immediately.

The history is intentionally not persisted. Closing/restarting Byte leaves only the actual saved customization, not an editor-history database.

## Reset behavior

Three reset scopes are available:

- Clear outfit
- Clear decor
- Reset look

Reset look restores:

- Byte
- default palette
- Meadow
- Curious
- Normal interaction
- no cosmetics
- no decorations

It preserves shell-specific presentation and placement state.

## Phase boundary

Phase 18 uses the complete finite local catalogs already shipped by Byte.

It does not add:

- cosmetic progression
- coins or currency
- marketplace/store
- paid items
- cloud inventory
- user-created cosmetics
- imported assets
- freeform habitat editing

Phase 19 now layers an optional Collection section onto this Studio without replacing any of the Phase 18 editing model.

See [COLLECTION.md](COLLECTION.md).

Next: **Phase 20 — Onboarding & Settings**.
