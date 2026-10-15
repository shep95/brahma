# spanish terminology and dynamic ui labels

this note goes with the review and completion of the spanish catalog
(`crates/ui-egui/src/i18n/es.tsv`). it already covered every key measured by
`cargo xtask i18n-coverage`; the terminology review aligned its wording with the vocabulary a
spanish-speaking image editor already knows, fixed mistranslations, and made the register
consistent. a later pass added the 1,276 runtime-only labels it still lacked. no proprietary
translation resources, fonts, screenshots or user documents are included: every translation was
written from the meaning of brahma's own english label.

## register and style

- **formal "usted"**, as in photoshop's spanish ui: «haga clic», «seleccione», «¿desea guardar…?».
  the catalog mixed «usted» and «tú»; every user-facing instruction now uses «usted».
- **sentence case** after the first word, except panel and tool names, which keep their capital
  as in photoshop: «panel carácter», «herramienta pluma».
- quotation marks are «angulares»: «{name}».
- neutral spanish, matching adobe's single spanish catalog (for example «coma flotante»).

## dynamic ui coverage

after merging the newer upstream catalog rows, the catalog has 4,010 entries. it covers the
complete italian runtime key set and retains two spanish profile-prompt labels that other
catalogs also use. the additional labels include filter and adjustment parameters and choices,
filter gallery names, camera raw, lighting effects, history and layers controls, metadata,
dialogs, status wrappers and system info labels. before this pass, those controls fell back to
english because many are assembled dynamically and cannot be found by the static coverage scan.

`spanish_covers_generated_filter_options_and_gallery_names` in `filter_dialog.rs` enumerates the
generated filter and adjustment surface and fails if a future label has no spanish row. a few
unused legacy labels are retained so the complete catalogs stay aligned.

## terminology

| english source | spanish ui | was |
|---|---|---|
| artboard | mesa de trabajo | lienzo (clashed with *canvas*) |
| pattern (fill, overlay, stamp) | motivo | patrón |
| preset | ajuste preestablecido | preajuste |
| keyboard shortcuts | métodos abreviados de teclado | atajos de teclado |
| grid | cuadrícula | rejilla |
| slice | sector | sección |
| plug-ins | plugins | complementos |
| scratch disks | discos de memoria virtual | discos temporales |
| puppet warp | deformación de posición libre | deformación de clavos |
| pin (puppet warp) | chincheta | pin |
| dodge / burn (tools) | sobreexponer / subexponer | **swapped** |
| color dodge / color burn | sobreexposición / subexposición de color | **swapped** |
| linear dodge (add) / linear burn | sobreexposición lineal (añadir) / subexposición lineal | |
| hard light / pin light / hard mix | luz fuerte / luz focal / mezcla definida | luz dura / luz puntual / mezcla dura |
| pass through (group) | pasar a través | herencia |
| clone stamp / pattern stamp | tampón de clonar / tampón de motivo | sello… |
| feather | calar | desvanecer (clashed with *fade*) |
| fade | transición | desvanecer |
| color range | gama de colores | intervalo de colores |
| select and mask | seleccionar y aplicar máscara | selección y máscara |
| selective color | corrección selectiva | color selectivo |
| shadows/highlights; highlights | sombras/iluminaciones; iluminaciones | sombras/luces; luces |
| color lookup | consulta de colores | búsqueda de color |
| content-aware fill / scale / move | relleno / escala según el contenido; mover con detección de contenido | relleno de contenido… |
| adaptive wide angle | gran angular adaptable | ángulo ancho adaptativo |
| constrain proportions | restringir proporciones | **construir** proporciones |
| render (filter menu) | interpretar | renderizar |
| smart sharpen | enfoque suavizado | enfoque inteligente |
| blend if | fusionar si | combinar si |
| knockout (blending options) | extraer | cobertura |
| lights / darks (camera raw) | claros / oscuros | tonos claros / tonos oscuros |
| master (hue/saturation) | todos | general |

menu names that are also actions keep photoshop's distinction: **vista** (the *view* menu, not
«ver»), **volver** (*revert*), **borrar** (*clear*), **cortar** (*trim*, unlike *crop* «recortar»),
**descubrir** (*reveal*), **purgar** (*purge*). product and technology names (photomerge, camera
raw, droplet, OpenType, srgb, wayland) are not translated.

## out of scope

engine-generated error details, status text and parameter help that do not pass through the ui
catalog remain english in every language (see
[`localization-parity.md`](localization-parity.md)). user document, layer and preset names and
automation command ids also remain unchanged. the completion was contributed by
@gorevyoneticisi with openai codex as an ai contributor under the repository's MIT or
apache-2.0 terms; a review by a native speaker is still welcome.
