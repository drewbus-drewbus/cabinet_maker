import { s as store_get, b as ensure_array_like, a as attr_class, e as escape_html, u as unsubscribe_stores } from "../../../chunks/index2.js";
import { n as nestingResults, p as project } from "../../../chunks/project.js";
import { b as selectedMaterialIndex, c as selectedSheetIndex } from "../../../chunks/ui.js";
function html(value) {
  var html2 = String(value);
  var open = "<!---->";
  return open + html2 + "<!---->";
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let svgContent = "";
    const currentGroup = store_get($$store_subs ??= {}, "$nestingResults", nestingResults)[store_get($$store_subs ??= {}, "$selectedMaterialIndex", selectedMaterialIndex)];
    const currentResult = currentGroup?.nesting_result;
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div><div class="flex items-center justify-between mb-4"><h1 class="text-2xl font-bold">Nesting</h1> <button class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded">Nest All Parts</button></div> `);
      if (store_get($$store_subs ??= {}, "$nestingResults", nestingResults).length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="flex gap-2 mb-4"><!--[-->`);
        const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$nestingResults", nestingResults));
        for (let i = 0, $$length = each_array.length; i < $$length; i++) {
          let group = each_array[i];
          $$renderer2.push(`<button${attr_class("px-3 py-1 text-xs rounded", void 0, {
            "bg-accent": store_get($$store_subs ??= {}, "$selectedMaterialIndex", selectedMaterialIndex) === i,
            "text-white": store_get($$store_subs ??= {}, "$selectedMaterialIndex", selectedMaterialIndex) === i,
            "bg-surface": store_get($$store_subs ??= {}, "$selectedMaterialIndex", selectedMaterialIndex) !== i,
            "hover:bg-surface-hover": store_get($$store_subs ??= {}, "$selectedMaterialIndex", selectedMaterialIndex) !== i
          })}>${escape_html(group.material_name)} (${escape_html(group.thickness)}")</button>`);
        }
        $$renderer2.push(`<!--]--></div> `);
        if (currentResult) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="flex items-center gap-4 mb-4"><div class="flex gap-1"><!--[-->`);
          const each_array_1 = ensure_array_like(currentResult.sheets);
          for (let i = 0, $$length = each_array_1.length; i < $$length; i++) {
            each_array_1[i];
            $$renderer2.push(`<button${attr_class("px-2 py-1 text-xs rounded", void 0, {
              "bg-surface-hover": store_get($$store_subs ??= {}, "$selectedSheetIndex", selectedSheetIndex) === i,
              "bg-surface": store_get($$store_subs ??= {}, "$selectedSheetIndex", selectedSheetIndex) !== i
            })}>Sheet ${escape_html(i + 1)}</button>`);
          }
          $$renderer2.push(`<!--]--></div> <div class="text-xs text-text-secondary">${escape_html(currentResult.sheet_count)} sheet(s) | ${escape_html(currentResult.overall_utilization.toFixed(1))}% utilization `);
          if (currentResult.unplaced.length > 0) {
            $$renderer2.push("<!--[-->");
            $$renderer2.push(`| <span class="text-error">${escape_html(currentResult.unplaced.length)} unplaced</span>`);
          } else {
            $$renderer2.push("<!--[!-->");
          }
          $$renderer2.push(`<!--]--></div></div> `);
          if (currentResult.sheets[store_get($$store_subs ??= {}, "$selectedSheetIndex", selectedSheetIndex)]) {
            $$renderer2.push("<!--[-->");
            const sheet = currentResult.sheets[store_get($$store_subs ??= {}, "$selectedSheetIndex", selectedSheetIndex)];
            $$renderer2.push(`<div class="flex gap-4 mb-4 text-xs text-text-secondary"><span>Parts: ${escape_html(sheet.parts.length)}</span> <span>Utilization: ${escape_html(sheet.utilization.toFixed(1))}%</span> <span>Waste: ${escape_html(sheet.waste_area.toFixed(1))} sq in</span></div>`);
          } else {
            $$renderer2.push("<!--[!-->");
          }
          $$renderer2.push(`<!--]--> <div class="bg-white rounded-lg p-4 overflow-auto max-h-[500px]">${html(svgContent)}</div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]-->`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<p class="text-text-secondary">Click "Nest All Parts" to arrange parts on sheets.</p>`);
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
