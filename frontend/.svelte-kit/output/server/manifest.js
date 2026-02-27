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
		client: {start:"_app/immutable/entry/start.DJCVKwwN.js",app:"_app/immutable/entry/app.FctBJ47D.js",imports:["_app/immutable/entry/start.DJCVKwwN.js","_app/immutable/chunks/DHqjY84K.js","_app/immutable/chunks/DUOIInB9.js","_app/immutable/chunks/CI19ujrK.js","_app/immutable/entry/app.FctBJ47D.js","_app/immutable/chunks/DUOIInB9.js","_app/immutable/chunks/CyNfHtry.js","_app/immutable/chunks/CI19ujrK.js","_app/immutable/chunks/B22MubrG.js","_app/immutable/chunks/IyXFpIzT.js","_app/immutable/chunks/D87RibR5.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
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
