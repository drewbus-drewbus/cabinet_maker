import { a as attr_class, s as store_get, e as escape_html, b as ensure_array_like, u as unsubscribe_stores } from "../../chunks/index2.js";
import { p as project } from "../../chunks/project.js";
import "../../chunks/history.js";
import "@sveltejs/kit/internal";
import "../../chunks/exports.js";
import "../../chunks/utils.js";
import "clsx";
import "@sveltejs/kit/internal/server";
import "../../chunks/root.js";
import "../../chunks/state.svelte.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let templates = [];
    function formatTemplateName(name) {
      return name.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
    }
    $$renderer2.push(`<div class="max-w-2xl mx-auto"><h1 class="text-3xl font-bold mb-2">Cabinet Maker</h1> <p class="text-text-secondary mb-8">Parametric cabinet design to CNC G-code pipeline</p> <div class="grid grid-cols-2 gap-4 mb-8"><button class="p-6 bg-surface hover:bg-surface-hover rounded-lg text-left transition-colors border border-border"><h3 class="text-lg font-semibold mb-1">New Project</h3> <p class="text-text-secondary text-sm">Start with a blank multi-cabinet project</p></button> <a href="/project"${attr_class("p-6 bg-surface rounded-lg text-left border border-border", void 0, {
      "opacity-50": !store_get($$store_subs ??= {}, "$project", project),
      "hover:bg-surface-hover": !!store_get($$store_subs ??= {}, "$project", project),
      "pointer-events-none": !store_get($$store_subs ??= {}, "$project", project)
    })}><h3 class="text-lg font-semibold mb-1">Continue</h3> <p class="text-text-secondary text-sm">`);
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`Resume "${escape_html(store_get($$store_subs ??= {}, "$project", project).project.name)}"`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`No project loaded`);
    }
    $$renderer2.push(`<!--]--></p></a></div> `);
    if (templates.length > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<h2 class="text-xl font-semibold mb-4">Templates</h2> <div class="grid grid-cols-3 gap-3"><!--[-->`);
      const each_array = ensure_array_like(templates);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let template = each_array[$$index];
        $$renderer2.push(`<button class="p-4 bg-surface hover:bg-surface-hover rounded-lg text-left transition-colors border border-border"><h3 class="text-sm font-semibold">${escape_html(formatTemplateName(template))}</h3> <p class="text-text-secondary text-xs mt-1">${escape_html(template)}.toml</p></button>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
