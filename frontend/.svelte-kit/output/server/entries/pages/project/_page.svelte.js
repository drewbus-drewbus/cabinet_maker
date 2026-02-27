import { s as store_get, c as attr, e as escape_html, u as unsubscribe_stores } from "../../../chunks/index2.js";
import { p as project, i as isDirty, d as isLoading, s as showToast } from "../../../chunks/project.js";
import { p as pushSnapshot } from "../../../chunks/history.js";
import { a as updateProject } from "../../../chunks/api.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    function handleUnitsChange(e) {
      const select = e.target;
      project.update((p) => {
        if (p) p.project.units = select.value;
        return p;
      });
      isDirty.set(true);
    }
    async function syncToBackend() {
      pushSnapshot();
      const p = store_get($$store_subs ??= {}, "$project", project);
      if (!p) return;
      try {
        isLoading.set(true);
        await updateProject(p);
        showToast("Project updated", "success");
      } catch (e) {
        showToast(`Error: ${e}`, "error");
      } finally {
        isLoading.set(false);
      }
    }
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="max-w-xl"><h1 class="text-2xl font-bold mb-6">Project Settings</h1> <div class="space-y-4"><div><label class="block text-sm text-text-secondary mb-1" for="project-name">Project Name</label> <input id="project-name" type="text"${attr("value", store_get($$store_subs ??= {}, "$project", project).project.name)} class="w-full px-3 py-2 bg-surface border border-border rounded text-text-primary text-sm focus:outline-none focus:border-accent"/></div> <div><label class="block text-sm text-text-secondary mb-1" for="project-units">Units</label> `);
      $$renderer2.select(
        {
          id: "project-units",
          value: store_get($$store_subs ??= {}, "$project", project).project.units,
          onchange: (e) => {
            handleUnitsChange(e);
            syncToBackend();
          },
          class: "w-full px-3 py-2 bg-surface border border-border rounded text-text-primary text-sm focus:outline-none focus:border-accent"
        },
        ($$renderer3) => {
          $$renderer3.option({ value: "inches" }, ($$renderer4) => {
            $$renderer4.push(`Inches`);
          });
          $$renderer3.option({ value: "millimeters" }, ($$renderer4) => {
            $$renderer4.push(`Millimeters`);
          });
        }
      );
      $$renderer2.push(`</div></div> <div class="mt-8"><h2 class="text-lg font-semibold mb-3">Summary</h2> <div class="grid grid-cols-2 gap-3 text-sm"><div class="p-3 bg-surface rounded border border-border"><div class="text-text-secondary">Cabinets</div> <div class="text-xl font-bold">${escape_html(store_get($$store_subs ??= {}, "$project", project).cabinets.length + (store_get($$store_subs ??= {}, "$project", project).cabinet ? 1 : 0))}</div></div> <div class="p-3 bg-surface rounded border border-border"><div class="text-text-secondary">Materials</div> <div class="text-xl font-bold">${escape_html(store_get($$store_subs ??= {}, "$project", project).materials.length + (store_get($$store_subs ??= {}, "$project", project).material ? 1 : 0))}</div></div> <div class="p-3 bg-surface rounded border border-border"><div class="text-text-secondary">Tools</div> <div class="text-xl font-bold">${escape_html(store_get($$store_subs ??= {}, "$project", project).tools.length)}</div></div> <div class="p-3 bg-surface rounded border border-border"><div class="text-text-secondary">Format</div> <div class="text-xl font-bold">${escape_html(store_get($$store_subs ??= {}, "$project", project).cabinet ? "Legacy" : "Multi")}</div></div></div></div></div>`);
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
