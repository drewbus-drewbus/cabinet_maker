import { s as store_get, e as escape_html, b as ensure_array_like, c as attr, u as unsubscribe_stores } from "../../../chunks/index2.js";
import { p as project, v as validationResult } from "../../../chunks/project.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let generatedSheets = [];
    let previewMaterialIdx = 0;
    let previewSheetIdx = 0;
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div><h1 class="text-2xl font-bold mb-6">Generate G-code</h1> <div class="grid grid-cols-2 gap-6 mb-6"><div class="p-4 bg-surface rounded-lg border border-border"><h2 class="text-lg font-semibold mb-3">Validation</h2> <button class="px-4 py-2 text-sm bg-surface-hover hover:bg-accent/50 rounded mb-3">Validate Project</button> `);
      if (store_get($$store_subs ??= {}, "$validationResult", validationResult)) {
        $$renderer2.push("<!--[-->");
        if (store_get($$store_subs ??= {}, "$validationResult", validationResult).errors.length === 0 && store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings.length === 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="text-success text-sm">All checks passed</div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (store_get($$store_subs ??= {}, "$validationResult", validationResult).errors.length > 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="text-error text-sm mb-2">${escape_html(store_get($$store_subs ??= {}, "$validationResult", validationResult).errors.length)} error(s)</div> <!--[-->`);
          const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$validationResult", validationResult).errors);
          for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
            let err = each_array[$$index];
            $$renderer2.push(`<div class="text-xs text-red-300 mb-1">${escape_html(JSON.stringify(err))}</div>`);
          }
          $$renderer2.push(`<!--]-->`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings.length > 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="text-warning text-sm mt-2 mb-2">${escape_html(store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings.length)} warning(s)</div> <!--[-->`);
          const each_array_1 = ensure_array_like(store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings);
          for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
            let warn = each_array_1[$$index_1];
            $$renderer2.push(`<div class="text-xs text-yellow-300 mb-1">${escape_html(JSON.stringify(warn))}</div>`);
          }
          $$renderer2.push(`<!--]-->`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]-->`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div class="p-4 bg-surface rounded-lg border border-border"><h2 class="text-lg font-semibold mb-3">Generate</h2> <button class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded mb-3">Generate G-code</button> `);
      if (generatedSheets.length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="text-sm text-success mb-2">Generated ${escape_html(generatedSheets.length)} file(s):</div> <ul class="text-xs text-text-secondary space-y-1"><!--[-->`);
        const each_array_2 = ensure_array_like(generatedSheets);
        for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
          let sheet = each_array_2[$$index_2];
          $$renderer2.push(`<li class="flex items-center gap-2"><span class="font-mono">${escape_html(sheet.filename)}</span> <button class="text-accent hover:underline">Download</button></li>`);
        }
        $$renderer2.push(`<!--]--></ul> <button class="mt-2 px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded">Download All</button>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></div> <div class="p-4 bg-surface rounded-lg border border-border"><div class="flex items-center justify-between mb-3"><h2 class="text-lg font-semibold">G-code Preview</h2> <div class="flex items-center gap-2"><label class="text-xs text-text-secondary">Material: <input type="number" min="0"${attr("value", previewMaterialIdx)} class="w-12 px-1 py-0.5 bg-bg border border-border rounded text-xs text-text-primary ml-1"/></label> <label class="text-xs text-text-secondary">Sheet: <input type="number" min="0"${attr("value", previewSheetIdx)} class="w-12 px-1 py-0.5 bg-bg border border-border rounded text-xs text-text-primary ml-1"/></label> <button class="px-3 py-1 text-xs bg-surface-hover hover:bg-accent/50 rounded">Preview</button></div></div> `);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<p class="text-text-secondary text-sm">Click "Preview" to see generated G-code.</p>`);
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
