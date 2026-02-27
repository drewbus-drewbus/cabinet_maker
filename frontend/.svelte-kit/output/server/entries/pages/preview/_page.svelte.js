import { a8 as ssr_context, s as store_get, b as ensure_array_like, e as escape_html, c as attr, a as attr_class, u as unsubscribe_stores } from "../../../chunks/index2.js";
import { p as project } from "../../../chunks/project.js";
import "clsx";
function onDestroy(fn) {
  /** @type {SSRContext} */
  ssr_context.r.on_destroy(fn);
}
function CabinetScene($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    onDestroy(() => {
    });
    $$renderer2.push(`<div class="w-full h-full min-h-[400px] rounded-lg overflow-hidden"></div>`);
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let panels = [];
    let exploded = false;
    let wireframe = false;
    let selectedPanel = null;
    const cabinets = () => {
      if (!store_get($$store_subs ??= {}, "$project", project)) return [];
      const cabs = [];
      if (store_get($$store_subs ??= {}, "$project", project).cabinet) cabs.push({
        name: store_get($$store_subs ??= {}, "$project", project).cabinet.name,
        index: 0
      });
      store_get($$store_subs ??= {}, "$project", project).cabinets.forEach((c, i) => cabs.push({ name: c.name, index: i }));
      return cabs;
    };
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="flex flex-col h-full"><div class="flex items-center justify-between mb-4"><h1 class="text-2xl font-bold">3D Preview</h1> <div class="flex items-center gap-3">`);
      if (cabinets().length > 1) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<select class="px-3 py-1 text-xs bg-surface border border-border rounded text-text-primary"><!--[-->`);
        const each_array = ensure_array_like(cabinets());
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let cab = each_array[$$index];
          $$renderer2.option({ value: cab.index }, ($$renderer3) => {
            $$renderer3.push(`${escape_html(cab.name)}`);
          });
        }
        $$renderer2.push(`<!--]--></select>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <label class="flex items-center gap-1 text-xs text-text-secondary cursor-pointer"><input type="checkbox"${attr("checked", exploded, true)} class="accent-accent"/> Exploded</label> <label class="flex items-center gap-1 text-xs text-text-secondary cursor-pointer"><input type="checkbox"${attr("checked", wireframe, true)} class="accent-accent"/> Wireframe</label></div></div> `);
      if (panels.length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="flex gap-6 flex-1 min-h-0"><div class="flex-1 bg-bg-secondary rounded-lg border border-border overflow-hidden">`);
        CabinetScene($$renderer2);
        $$renderer2.push(`<!----></div> <div class="w-48 flex-shrink-0 overflow-auto"><h3 class="text-sm font-semibold mb-2">Panels</h3> <div class="space-y-1"><!--[-->`);
        const each_array_1 = ensure_array_like(panels);
        for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
          let panel = each_array_1[$$index_1];
          $$renderer2.push(`<button${attr_class("w-full text-left px-2 py-1 text-xs rounded transition-colors", void 0, {
            "bg-accent": selectedPanel === panel.label,
            "text-white": selectedPanel === panel.label,
            "hover:bg-surface-hover": selectedPanel !== panel.label
          })}><div class="font-medium">${escape_html(panel.label)}</div> <div class="text-text-secondary">${escape_html(panel.width.toFixed(2))} x ${escape_html(panel.height.toFixed(2))} x ${escape_html(panel.depth.toFixed(2))}</div></button>`);
        }
        $$renderer2.push(`<!--]--></div></div></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<p class="text-text-secondary">No cabinet selected or no parts generated.</p>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<div class="text-text-secondary"><p>No project loaded. <a href="/" class="text-accent hover:underline">Go home</a> to create or open one.</p></div>`);
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
