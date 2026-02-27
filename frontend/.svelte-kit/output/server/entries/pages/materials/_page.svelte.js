import { s as store_get, b as ensure_array_like, a as attr_class, e as escape_html, c as attr, u as unsubscribe_stores, d as derived } from "../../../chunks/index2.js";
import { p as project, s as showToast, i as isDirty } from "../../../chunks/project.js";
import { p as pushSnapshot } from "../../../chunks/history.js";
import { a as updateProject } from "../../../chunks/api.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let selectedMaterialIdx = null;
    let selectedToolIdx = null;
    async function syncProject() {
      pushSnapshot();
      const p = store_get($$store_subs ??= {}, "$project", project);
      if (!p) return;
      try {
        await updateProject(p);
      } catch (e) {
        showToast(`Error: ${e}`, "error");
      }
    }
    function updateMaterialField(field, value) {
      project.update((p) => {
        return p;
      });
      isDirty.set(true);
    }
    const selectedMaterial = derived(() => store_get($$store_subs ??= {}, "$project", project) && selectedMaterialIdx !== null ? store_get($$store_subs ??= {}, "$project", project).materials[selectedMaterialIdx] : null);
    const selectedTool = derived(() => store_get($$store_subs ??= {}, "$project", project) && selectedToolIdx !== null ? store_get($$store_subs ??= {}, "$project", project).tools[selectedToolIdx] : null);
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
      if (selectedMaterial()) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="p-4 bg-surface rounded border border-border space-y-3"><div><label class="block text-xs text-text-secondary mb-1">Name</label> <input type="text"${attr("value", selectedMaterial().name)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div class="grid grid-cols-2 gap-3"><div><label class="block text-xs text-text-secondary mb-1">Thickness</label> <input type="number" step="0.125"${attr("value", selectedMaterial().thickness)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Type</label> `);
        $$renderer2.select(
          {
            value: selectedMaterial().material_type,
            onchange: (e) => {
              updateMaterialField("material_type", e.target.value);
              syncProject();
            },
            class: "w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
          },
          ($$renderer3) => {
            $$renderer3.option({ value: "plywood" }, ($$renderer4) => {
              $$renderer4.push(`Plywood`);
            });
            $$renderer3.option({ value: "hardwood" }, ($$renderer4) => {
              $$renderer4.push(`Hardwood`);
            });
            $$renderer3.option({ value: "mdf" }, ($$renderer4) => {
              $$renderer4.push(`MDF`);
            });
            $$renderer3.option({ value: "melamine" }, ($$renderer4) => {
              $$renderer4.push(`Melamine`);
            });
          }
        );
        $$renderer2.push(`</div> <div><label class="block text-xs text-text-secondary mb-1">Sheet Width</label> <input type="number" step="1"${attr("value", selectedMaterial().sheet_width)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Sheet Length</label> <input type="number" step="1"${attr("value", selectedMaterial().sheet_length)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div></div></div>`);
      } else {
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
      if (selectedTool()) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="p-4 bg-surface rounded border border-border space-y-3"><div><label class="block text-xs text-text-secondary mb-1">Description</label> <input type="text"${attr("value", selectedTool().description)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div class="grid grid-cols-2 gap-3"><div><label class="block text-xs text-text-secondary mb-1">Number</label> <input type="number" min="1"${attr("value", selectedTool().number)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Diameter</label> <input type="number" step="0.0625"${attr("value", selectedTool().diameter)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Flutes</label> <input type="number" min="1" max="8"${attr("value", selectedTool().flutes)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Cutting Length</label> <input type="number" step="0.125"${attr("value", selectedTool().cutting_length)} class="w-full px-3 py-2 bg-bg border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div></div></div>`);
      } else {
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
