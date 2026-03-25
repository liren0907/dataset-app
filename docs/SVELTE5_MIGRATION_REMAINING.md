# Svelte 5 Migration — Complete

Migration from Svelte 4 to Svelte 5 completed on 2026-03-22.

## Summary

- **100 `.svelte` files** migrated to Svelte 5 runes (`$props`, `$state`, `$derived`, `$effect`)
- **10 store files** converted from `writable`/`derived` to `$state` runes (renamed to `.svelte.ts`)
- **Build**: Passes cleanly (`yarn build`)
- **Runtime testing**: Passed

## All items completed

1. ~~Stores converted to runes~~ — All 10 store files converted to `$state`/`$derived` runes
2. ~~Pre-existing TypeScript errors~~ — Legacy files moved to `deprecated/`, remaining errors fixed
3. ~~`svelte-dnd-action` event syntax~~ — Runtime tested, works correctly
4. ~~Runtime testing~~ — All flows passing
5. ~~Legacy root store cleanup~~ — `src/store.js` and `src/funcs/` moved to `deprecated/`
