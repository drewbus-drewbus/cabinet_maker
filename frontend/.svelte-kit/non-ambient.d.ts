
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	export interface AppTypes {
		RouteId(): "/" | "/cabinets" | "/cutlist" | "/generate" | "/materials" | "/nesting" | "/preview" | "/project";
		RouteParams(): {
			
		};
		LayoutParams(): {
			"/": Record<string, never>;
			"/cabinets": Record<string, never>;
			"/cutlist": Record<string, never>;
			"/generate": Record<string, never>;
			"/materials": Record<string, never>;
			"/nesting": Record<string, never>;
			"/preview": Record<string, never>;
			"/project": Record<string, never>
		};
		Pathname(): "/" | "/cabinets" | "/cutlist" | "/generate" | "/materials" | "/nesting" | "/preview" | "/project";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): string & {};
	}
}