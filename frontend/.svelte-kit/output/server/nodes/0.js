

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "prerender": true,
  "ssr": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.ImHCqvwQ.js","_app/immutable/chunks/CyNfHtry.js","_app/immutable/chunks/DUOIInB9.js","_app/immutable/chunks/B22MubrG.js","_app/immutable/chunks/BI3IV-ao.js","_app/immutable/chunks/C2eLq_D0.js","_app/immutable/chunks/C0zNEv0Q.js","_app/immutable/chunks/D7iecON0.js","_app/immutable/chunks/D5y8zx2n.js","_app/immutable/chunks/DHqjY84K.js","_app/immutable/chunks/CI19ujrK.js","_app/immutable/chunks/B-0l54sc.js","_app/immutable/chunks/neMrXvPf.js","_app/immutable/chunks/IyXFpIzT.js","_app/immutable/chunks/DfOa05yV.js"];
export const stylesheets = ["_app/immutable/assets/0.D5afwOJM.css"];
export const fonts = [];
