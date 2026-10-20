// brahma's decode worker: a second copy of the app's WebAssembly module that only decodes
// photos, so a large JPEG never freezes the page (the window keeps drawing while it opens).
// The page sends {id, name, bytes, js, wasm}: the module's generated JS and wasm URLs (hashed
// by the build), the file's name and bytes. The reply is {id, out} with the decoded image as
// one byte envelope (brahma_io::decoded), or {id, error}.
let ready = null;
let mod = null;
self.onmessage = async (e) => {
  const { id, name, bytes, js, wasm } = e.data;
  try {
    if (!ready) {
      ready = (async () => {
        mod = await import(js);
        await mod.default({ module_or_path: wasm });
      })();
    }
    await ready;
    const out = mod.worker_decode(name, new Uint8Array(bytes));
    self.postMessage({ id, out }, [out.buffer]);
  } catch (err) {
    self.postMessage({ id, error: String(err && err.message ? err.message : err) });
  }
};
