import { s as store_get, b as ensure_array_like, a as attr_class, e as escape_html, c as attr, u as unsubscribe_stores, d as derived } from "../../../chunks/index2.js";
import { p as project, i as isDirty, s as showToast } from "../../../chunks/project.js";
import { a as selectedCabinetIndex } from "../../../chunks/ui.js";
import { p as pushSnapshot } from "../../../chunks/history.js";
import { u as updateCabinet } from "../../../chunks/api.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let previewParts = [];
    const cabinetTypes = [
      { value: "basic_box", label: "Basic Box" },
      { value: "base_cabinet", label: "Base Cabinet" },
      { value: "wall_cabinet", label: "Wall Cabinet" },
      { value: "tall_cabinet", label: "Tall Cabinet" },
      { value: "sink_base", label: "Sink Base" },
      { value: "drawer_bank", label: "Drawer Bank" }
    ];
    async function handleUpdateCabinet() {
      pushSnapshot();
      const idx = store_get($$store_subs ??= {}, "$selectedCabinetIndex", selectedCabinetIndex);
      const p = store_get($$store_subs ??= {}, "$project", project);
      if (idx === null || !p || idx >= p.cabinets.length) return;
      try {
        await updateCabinet(idx, p.cabinets[idx]);
        isDirty.set(true);
      } catch (e) {
        showToast(`Error: ${e}`, "error");
      }
    }
    function updateField(field, value) {
      const idx = store_get($$store_subs ??= {}, "$selectedCabinetIndex", selectedCabinetIndex);
      project.update((p) => {
        if (p && idx !== null && idx < p.cabinets.length) {
          p.cabinets[idx][field] = value;
        }
        return p;
      });
      isDirty.set(true);
    }
    function getSelectedCabinet() {
      const idx = store_get($$store_subs ??= {}, "$selectedCabinetIndex", selectedCabinetIndex);
      const p = store_get($$store_subs ??= {}, "$project", project);
      if (idx === null || !p || idx >= p.cabinets.length) return null;
      return p.cabinets[idx];
    }
    const selectedCab = derived(getSelectedCabinet);
    const showToeKick = derived(() => selectedCab()?.cabinet_type === "base_cabinet" || selectedCab()?.cabinet_type === "tall_cabinet" || selectedCab()?.cabinet_type === "drawer_bank");
    const showDrawers = derived(() => selectedCab()?.cabinet_type === "drawer_bank");
    const showStretchers = derived(() => selectedCab()?.cabinet_type === "sink_base");
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="flex gap-6 h-full"><div class="w-56 flex-shrink-0"><div class="flex items-center justify-between mb-3"><h2 class="text-lg font-semibold">Cabinets</h2> <button class="px-2 py-1 text-xs bg-accent hover:bg-accent/80 text-white rounded">+ Add</button></div> <div class="space-y-1"><!--[-->`);
      const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$project", project).cabinets);
      for (let i = 0, $$length = each_array.length; i < $$length; i++) {
        let entry = each_array[i];
        $$renderer2.push(`<div${attr_class("w-full flex items-center justify-between px-3 py-2 text-sm rounded transition-colors text-left cursor-pointer", void 0, {
          "bg-surface": store_get($$store_subs ??= {}, "$selectedCabinetIndex", selectedCabinetIndex) === i,
          "text-accent": store_get($$store_subs ??= {}, "$selectedCabinetIndex", selectedCabinetIndex) === i,
          "hover:bg-surface-hover": store_get($$store_subs ??= {}, "$selectedCabinetIndex", selectedCabinetIndex) !== i
        })} role="button" tabindex="0"><div><div class="font-medium">${escape_html(entry.name)}</div> <div class="text-xs text-text-secondary">${escape_html(entry.cabinet_type.replace("_", " "))}</div></div> <button class="text-text-secondary hover:text-error text-xs px-1">x</button></div>`);
      }
      $$renderer2.push(`<!--]--></div> `);
      if (store_get($$store_subs ??= {}, "$project", project).cabinets.length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<button class="mt-4 w-full px-3 py-2 text-xs bg-surface hover:bg-surface-hover rounded text-center">Generate All Parts</button>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div class="flex-1 overflow-auto">`);
      if (selectedCab()) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<h2 class="text-lg font-semibold mb-4">Edit: ${escape_html(selectedCab().name)}</h2> <div class="grid grid-cols-2 gap-4 max-w-lg"><div class="col-span-2"><label class="block text-xs text-text-secondary mb-1">Name</label> <input type="text"${attr("value", selectedCab().name)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div class="col-span-2"><label class="block text-xs text-text-secondary mb-1">Type</label> `);
        $$renderer2.select(
          {
            value: selectedCab().cabinet_type,
            onchange: (e) => {
              updateField("cabinet_type", e.target.value);
              handleUpdateCabinet();
            },
            class: "w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"
          },
          ($$renderer3) => {
            $$renderer3.push(`<!--[-->`);
            const each_array_1 = ensure_array_like(cabinetTypes);
            for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
              let ct = each_array_1[$$index_1];
              $$renderer3.option({ value: ct.value }, ($$renderer4) => {
                $$renderer4.push(`${escape_html(ct.label)}`);
              });
            }
            $$renderer3.push(`<!--]-->`);
          }
        );
        $$renderer2.push(`</div> <div><label class="block text-xs text-text-secondary mb-1">Width</label> <input type="number" step="0.25"${attr("value", selectedCab().width)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Height</label> <input type="number" step="0.25"${attr("value", selectedCab().height)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Depth</label> <input type="number" step="0.25"${attr("value", selectedCab().depth)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Shelves</label> <input type="number" min="0" max="20"${attr("value", selectedCab().shelf_count)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Material Thickness</label> <input type="number" step="0.125"${attr("value", selectedCab().material_thickness)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Back Thickness</label> <input type="number" step="0.125"${attr("value", selectedCab().back_thickness)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer"><input type="checkbox"${attr("checked", selectedCab().has_back, true)} class="accent-accent"/> Has Back Panel</label></div> `);
        if (showToeKick()) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="col-span-2 border-t border-border pt-3 mt-2"><h3 class="text-sm font-semibold mb-2">Toe Kick</h3> <div class="grid grid-cols-2 gap-3"><div><label class="block text-xs text-text-secondary mb-1">Height</label> <input type="number" step="0.25"${attr("value", selectedCab().toe_kick?.height ?? 4)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Setback</label> <input type="number" step="0.25"${attr("value", selectedCab().toe_kick?.setback ?? 3)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div></div></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (showDrawers()) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="col-span-2 border-t border-border pt-3 mt-2"><h3 class="text-sm font-semibold mb-2">Drawers</h3> <div class="grid grid-cols-2 gap-3"><div><label class="block text-xs text-text-secondary mb-1">Count</label> <input type="number" min="1" max="10"${attr("value", selectedCab().drawers?.count ?? 4)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="block text-xs text-text-secondary mb-1">Slide Clearance</label> <input type="number" step="0.125"${attr("value", selectedCab().drawers?.slide_clearance ?? 0.5)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div></div></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (showStretchers()) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="col-span-2 border-t border-border pt-3 mt-2"><h3 class="text-sm font-semibold mb-2">Stretchers</h3> <div class="grid grid-cols-2 gap-3"><div><label class="block text-xs text-text-secondary mb-1">Front Width</label> <input type="number" step="0.25"${attr("value", selectedCab().stretchers?.front_width ?? 4)} class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary focus:outline-none focus:border-accent"/></div> <div><label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer mt-5"><input type="checkbox"${attr("checked", selectedCab().stretchers?.has_rear ?? true, true)} class="accent-accent"/> Has Rear Stretcher</label></div></div></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--></div> `);
        if (previewParts.length > 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="mt-6 border-t border-border pt-4"><h3 class="text-sm font-semibold mb-2">Parts Preview (${escape_html(previewParts.length)} parts)</h3> <div class="overflow-auto max-h-64"><table class="w-full text-xs"><thead><tr class="text-text-secondary border-b border-border"><th class="py-1 text-left">Label</th><th class="py-1 text-right">Width</th><th class="py-1 text-right">Height</th><th class="py-1 text-right">Qty</th><th class="py-1 text-right">Ops</th></tr></thead><tbody><!--[-->`);
          const each_array_2 = ensure_array_like(previewParts);
          for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
            let part = each_array_2[$$index_2];
            $$renderer2.push(`<tr class="border-b border-border/50"><td class="py-1">${escape_html(part.label)}</td><td class="py-1 text-right">${escape_html(part.rect.width.toFixed(3))}"</td><td class="py-1 text-right">${escape_html(part.rect.height.toFixed(3))}"</td><td class="py-1 text-right">${escape_html(part.quantity)}</td><td class="py-1 text-right">${escape_html(part.operations.length)}</td></tr>`);
          }
          $$renderer2.push(`<!--]--></tbody></table></div></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]-->`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<div class="flex items-center justify-center h-full text-text-secondary"><p>Select a cabinet from the list or add a new one</p></div>`);
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
