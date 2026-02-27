import { s as store_get, e as escape_html, b as ensure_array_like, u as unsubscribe_stores, d as derived } from "../../../chunks/index2.js";
import { p as project, v as validationResult, f as cutlistRows } from "../../../chunks/project.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let sortColumn = "cabinet";
    const sortedRows = derived(() => [
      ...store_get($$store_subs ??= {}, "$cutlistRows", cutlistRows)
    ].sort((a, b) => {
      const av = a[sortColumn];
      const bv = b[sortColumn];
      const cmp = typeof av === "string" ? av.localeCompare(bv) : av - bv;
      return cmp;
    }));
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div><div class="flex items-center justify-between mb-4"><h1 class="text-2xl font-bold">Cut List</h1> <div class="flex gap-2"><button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded">Refresh</button> <button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded">Export CSV</button> <button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover rounded">Export JSON</button></div></div> `);
      if (store_get($$store_subs ??= {}, "$validationResult", validationResult)) {
        $$renderer2.push("<!--[-->");
        if (store_get($$store_subs ??= {}, "$validationResult", validationResult).errors.length > 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="mb-4 p-3 bg-red-900/30 border border-red-600/50 rounded"><h3 class="text-sm font-semibold text-error mb-1">Errors (${escape_html(store_get($$store_subs ??= {}, "$validationResult", validationResult).errors.length)})</h3> <!--[-->`);
          const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$validationResult", validationResult).errors);
          for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
            let err = each_array[$$index];
            $$renderer2.push(`<div class="text-xs text-red-300">${escape_html(JSON.stringify(err))}</div>`);
          }
          $$renderer2.push(`<!--]--></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings.length > 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="mb-4 p-3 bg-yellow-900/30 border border-yellow-600/50 rounded"><h3 class="text-sm font-semibold text-warning mb-1">Warnings (${escape_html(store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings.length)})</h3> <!--[-->`);
          const each_array_1 = ensure_array_like(store_get($$store_subs ??= {}, "$validationResult", validationResult).warnings);
          for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
            let warn = each_array_1[$$index_1];
            $$renderer2.push(`<div class="text-xs text-yellow-300">${escape_html(JSON.stringify(warn))}</div>`);
          }
          $$renderer2.push(`<!--]--></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]-->`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> `);
      if (sortedRows().length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="overflow-auto"><table class="w-full text-sm"><thead><tr class="text-text-secondary border-b border-border"><!--[-->`);
        const each_array_2 = ensure_array_like([
          "cabinet",
          "label",
          "material",
          "width",
          "height",
          "thickness",
          "quantity"
        ]);
        for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
          let col = each_array_2[$$index_2];
          $$renderer2.push(`<th class="py-2 px-2 text-left cursor-pointer hover:text-text-primary">${escape_html(col.charAt(0).toUpperCase() + col.slice(1))} `);
          if (sortColumn === col) {
            $$renderer2.push("<!--[-->");
            $$renderer2.push(`${escape_html(" ^")}`);
          } else {
            $$renderer2.push("<!--[!-->");
          }
          $$renderer2.push(`<!--]--></th>`);
        }
        $$renderer2.push(`<!--]--><th class="py-2 px-2 text-left">Operations</th></tr></thead><tbody><!--[-->`);
        const each_array_3 = ensure_array_like(sortedRows());
        for (let $$index_3 = 0, $$length = each_array_3.length; $$index_3 < $$length; $$index_3++) {
          let row = each_array_3[$$index_3];
          $$renderer2.push(`<tr class="border-b border-border/30 hover:bg-surface/50"><td class="py-1 px-2">${escape_html(row.cabinet)}</td><td class="py-1 px-2 font-medium">${escape_html(row.label)}</td><td class="py-1 px-2">${escape_html(row.material)}</td><td class="py-1 px-2 text-right font-mono">${escape_html(row.width.toFixed(3))}"</td><td class="py-1 px-2 text-right font-mono">${escape_html(row.height.toFixed(3))}"</td><td class="py-1 px-2 text-right font-mono">${escape_html(row.thickness.toFixed(3))}"</td><td class="py-1 px-2 text-center">${escape_html(row.quantity)}</td><td class="py-1 px-2 text-xs text-text-secondary">${escape_html(row.operations.join(", ") || "-")}</td></tr>`);
        }
        $$renderer2.push(`<!--]--></tbody></table></div> <div class="mt-3 text-xs text-text-secondary">${escape_html(sortedRows().length)} parts total</div>`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<p class="text-text-secondary">No parts generated yet. Go to Cabinets and click "Generate All Parts".</p>`);
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
