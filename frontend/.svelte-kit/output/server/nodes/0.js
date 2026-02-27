

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "prerender": true,
  "ssr": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.C857yhBe.js","_app/immutable/chunks/pWOMhstZ.js","_app/immutable/chunks/BepynHdk.js","_app/immutable/chunks/CaLNNbA5.js","_app/immutable/chunks/DWMYr-GH.js","_app/immutable/chunks/9pRRSWtK.js","_app/immutable/chunks/BfDX1O-d.js","_app/immutable/chunks/DkQQvypr.js","_app/immutable/chunks/r8tgtEiy.js","_app/immutable/chunks/BkaQf1bU.js","_app/immutable/chunks/Cuutpi9m.js","_app/immutable/chunks/DEoDEiov.js","_app/immutable/chunks/Gq__uSar.js","_app/immutable/chunks/EnNr4Sxw.js","_app/immutable/chunks/DiomP2yI.js"];
export const stylesheets = ["_app/immutable/assets/0.zC9y-FbB.css"];
export const fonts = [];
