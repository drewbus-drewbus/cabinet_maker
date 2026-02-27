export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.ookMOhHT.js",app:"_app/immutable/entry/app.u5zubr-l.js",imports:["_app/immutable/entry/start.ookMOhHT.js","_app/immutable/chunks/BkaQf1bU.js","_app/immutable/chunks/BepynHdk.js","_app/immutable/chunks/Cuutpi9m.js","_app/immutable/entry/app.u5zubr-l.js","_app/immutable/chunks/BepynHdk.js","_app/immutable/chunks/pWOMhstZ.js","_app/immutable/chunks/Cuutpi9m.js","_app/immutable/chunks/CaLNNbA5.js","_app/immutable/chunks/EnNr4Sxw.js","_app/immutable/chunks/D0cirAwm.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js'))
		],
		remotes: {
			
		},
		routes: [
			
		],
		prerendered_routes: new Set(["/","/cabinets","/cutlist","/generate","/materials","/nesting","/preview","/project"]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
