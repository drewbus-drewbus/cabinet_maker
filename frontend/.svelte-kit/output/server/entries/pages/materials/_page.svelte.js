import { s as store_get, b as ensure_array_like, a as attr_class, e as escape_html, u as unsubscribe_stores } from "../../../chunks/index2.js";
import { p as project } from "../../../chunks/project.js";
import "../../../chunks/history.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let selectedMaterialIdx = null;
    let selectedToolIdx = null;
    store_get($$store_subs ??= {}, "$project", project) && selectedMaterialIdx !== null ? store_get($$store_subs ??= {}, "$project", project).materials[selectedMaterialIdx] : null;
    store_get($$store_subs ??= {}, "$project", project) && selectedToolIdx !== null ? store_get($$store_subs ??= {}, "$project", project).tools[selectedToolIdx] : null;
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="grid grid-cols-2 gap-8"><div><div class="flex items-center justify-between mb-3"><h2 class="text-lg font-semibold">Materials</h2> <button class="px-2 py-1 text-xs bg-accent hover:bg-accent/80 text-white rounded">+ Add</button></div> <div class="space-y-1 mb-4"><!--[-->`);
      const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$project", project).materials);
      for (let i = 0, $$length = each_array.length; i < $$length; i++) {
        let mat = each_array[i];
        $$renderer2.push(`<div${attr_class("w-full flex items-center justify-between px-3 py-2 text-sm rounded text-left cursor-pointer", void 0, {
          "bg-surface": selectedMaterialIdx === i,
          "hover:bg-surface-hover": selectedMaterialIdx !== i
        })} role="button" tabindex="0"><div><div class="font-medium">${escape_html(mat.name)}</div> <div class="text-xs text-text-secondary">${escape_html(mat.thickness)}" ${escape_html(mat.material_type)}</div></div> <button class="text-text-secondary hover:text-error text-xs px-1">x</button></div>`);
      }
      $$renderer2.push(`<!--]--></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div><div class="flex items-center justify-between mb-3"><h2 class="text-lg font-semibold">Tools</h2> <button class="px-2 py-1 text-xs bg-accent hover:bg-accent/80 text-white rounded">+ Add</button></div> <div class="space-y-1 mb-4"><!--[-->`);
      const each_array_1 = ensure_array_like(store_get($$store_subs ??= {}, "$project", project).tools);
      for (let i = 0, $$length = each_array_1.length; i < $$length; i++) {
        let tool = each_array_1[i];
        $$renderer2.push(`<div${attr_class("w-full flex items-center justify-between px-3 py-2 text-sm rounded text-left cursor-pointer", void 0, {
          "bg-surface": selectedToolIdx === i,
          "hover:bg-surface-hover": selectedToolIdx !== i
        })} role="button" tabindex="0"><div><div class="font-medium">T${escape_html(tool.number)}: ${escape_html(tool.description)}</div> <div class="text-xs text-text-secondary">${escape_html(tool.diameter)}" ${escape_html(tool.tool_type)}</div></div> <button class="text-text-secondary hover:text-error text-xs px-1">x</button></div>`);
      }
      $$renderer2.push(`<!--]--></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></div>`);
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
