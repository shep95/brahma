# italian terminology and dynamic ui labels

this note goes with the completion of the italian catalog (`crates/ui-egui/src/i18n/it.tsv`).
the catalog already covered every key `cargo xtask i18n-coverage` measures, but it lacked 1,287
labels that only exist at runtime and that other catalogs (brazilian portuguese, dutch, czech)
already translate: filter and adjustment parameter names, camera raw panels, lighting effects,
history, layers, path and channel menu labels, metadata fields, status messages, system info and
dialog text. in italian those showed in english.
they are now translated, and a review pass fixed mistranslations and inconsistent terms in the
existing entries. translations are written from the meaning of brahma's english labels; no
proprietary translation resources, fonts, screenshots or user documents are included.

## register and style

commands are imperatives in the second person singular (*elimina*, *applica*, *crea
pacchetto*); text addresses the user as *tu*. only the first word is capitalised, except in
command names joined by a slash, which keep both capitals (*luminosità/contrasto*,
*HSL/scala di grigio*). decimals use a comma.

## terminology

| english source | italian | note |
|---|---|---|
| layer | livello | |
| canvas | quadro | also in *come il quadro*, *applica ai limiti del quadro* |
| preset | predefinito | *salva come predefinito* |
| default(s) | predefinito / predefiniti | never *default* or *valori predefiniti* |
| stroke | traccia | contour and painterly strokes alike (*tracce angolate*, *direzione traccia*); a dash stays *tratto* |
| sharpen | contrasta / contrasto | also in camera raw (*quantità contrasto*) |
| noise | disturbo | *riduzione disturbo di luminanza*, *riduzione disturbo cromatico* |
| highlights | luci | pairs with *ombre* |
| snapshot | istantanea | |
| smart filters | filtri avanzati | |
| pattern | pattern | *timbro pattern* |
| gradient stops | punti della sfumatura | |
| spot color | tinta piatta | *canale tinta piatta* |
| click | clic | *fai clic*, *{key}-clic*, *doppio clic* |
| keyboard shortcut | scorciatoia da tastiera | |
| footage | filmato | *interpreta filmato*, *sostituisci filmato* |
| facet | sfaccetta | |
| SemiBold / ExtraBold / ExtraLight | semigrassetto / extragrassetto / extraleggero | one spelling for every weight |
| (color) mode | metodo | *cambia metodo condizionale*, never *modo* |
| trap | trapping | the prepress term; *sovrastampa* is overprint, a different thing |
| HDR toning | mappatura tonale HDR | tone mapping; *viraggio* stays for colour toning (*viraggio separato*) |
| bevel & emboss | smusso e rilievo | pairs with *rilievo* (emboss) |
| linked (smart object) | oggetto collegato | *inserisci oggetto collegato*, *converti in oggetto collegato* |

## native review

2026-10-12: a native italian speaker read the catalog and fixed mistranslations (trap, HDR
toning, display text, vine color, override action "save" commands) and unnatural phrasing (bevel &
emboss, east asian / middle eastern features, linked smart objects, bit depth labels).

## not covered

engine error and status messages, user document, layer and preset names, and automation
command ids stay in english, as in every language. a few dynamic labels the other catalogs carry
(`Ask Before Saving Saved Embedded`, `Clear Selection`, the `Heal` family) are not used by the
current code; they are translated to stay aligned with the other catalogs.

## validation

- `italian_covers_generated_filter_options_and_gallery_names` (`crates/ui-egui/src/filter_dialog.rs`)
  enumerates every filter and adjustment parameter, symbolic choice and gallery name, like the
  korean test, and requires an italian row for each. with the catalog on `main` before this change
  it lists 388 missing labels; now none.
- `cargo test -p brahma-ui-egui --lib`: the catalog tests pass. two layout tests,
  `type_controls_fill_the_panel_at_every_width_and_scale` and
  `properties_edit_shown_metrics_and_paragraphs_with_a_non_type_primary`, fail on `main` as well
  without this change.
- `cargo xtask i18n-coverage`: 1,726 / 1,726 for `it`.
- the offscreen `snapshot` example with `BRAHMA_LOCALE=it` rendered lighting effects, the
  camera raw filter and wind with italian labels, without missing glyphs or clipped text.
