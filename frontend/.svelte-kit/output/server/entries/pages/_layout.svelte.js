import "clsx";
import { a as attr_class, s as store_get, e as escape_html, b as ensure_array_like, c as attr, u as unsubscribe_stores } from "../../chunks/index2.js";
import { p as page } from "../../chunks/index3.js";
import { p as project, c as cabinetCount, t as totalPartCount, i as isDirty, a as projectPath, b as cachedParts, v as validationResult, d as isLoading, e as toasts } from "../../chunks/project.js";
import { s as sidebarCollapsed } from "../../chunks/ui.js";
import { c as canUndo, a as canRedo } from "../../chunks/history.js";
function Sidebar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    const navItems = [
      { href: "/", label: "Home", icon: "H" },
      { href: "/project", label: "Project", icon: "P" },
      { href: "/cabinets", label: "Cabinets", icon: "C" },
      { href: "/materials", label: "Materials", icon: "M" },
      { href: "/cutlist", label: "Cut List", icon: "L" },
      { href: "/nesting", label: "Nesting", icon: "N" },
      { href: "/preview", label: "3D Preview", icon: "3" },
      { href: "/generate", label: "Generate", icon: "G" }
    ];
    $$renderer2.push(`<nav${attr_class("flex flex-col bg-bg-secondary border-r border-border h-full transition-all duration-200", void 0, {
      "w-48": !store_get($$store_subs ??= {}, "$sidebarCollapsed", sidebarCollapsed),
      "w-12": store_get($$store_subs ??= {}, "$sidebarCollapsed", sidebarCollapsed)
    })}><div class="flex items-center justify-between p-3 border-b border-border">`);
    if (!store_get($$store_subs ??= {}, "$sidebarCollapsed", sidebarCollapsed)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="font-bold text-sm text-text-primary">Cabinet Maker</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <button class="text-text-secondary hover:text-text-primary text-xs px-1">${escape_html(store_get($$store_subs ??= {}, "$sidebarCollapsed", sidebarCollapsed) ? ">" : "<")}</button></div> <div class="flex-1 py-2"><!--[-->`);
    const each_array = ensure_array_like(navItems);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let item = each_array[$$index];
      const isActive = page.url.pathname === item.href;
      $$renderer2.push(`<a${attr("href", item.href)}${attr_class("flex items-center gap-2 px-3 py-2 text-sm transition-colors", void 0, {
        "bg-surface": isActive,
        "text-accent": isActive,
        "text-text-secondary": !isActive,
        "hover:bg-surface-hover": !isActive,
        "hover:text-text-primary": !isActive
      })}><span class="w-5 h-5 flex items-center justify-center text-xs font-mono font-bold rounded bg-border">${escape_html(item.icon)}</span> `);
      if (!store_get($$store_subs ??= {}, "$sidebarCollapsed", sidebarCollapsed)) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<span>${escape_html(item.label)}</span>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></a>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    if (!store_get($$store_subs ??= {}, "$sidebarCollapsed", sidebarCollapsed) && store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="p-3 border-t border-border text-xs text-text-secondary space-y-1"><div>Cabinets: ${escape_html(store_get($$store_subs ??= {}, "$cabinetCount", cabinetCount))}</div> <div>Parts: ${escape_html(store_get($$store_subs ??= {}, "$totalPartCount", totalPartCount))}</div></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></nav>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function TopBar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    $$renderer2.push(`<input type="file" accept=".toml" class="hidden"/> <header class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border"><div class="flex items-center gap-2"><button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors">New</button> <button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors">Open</button> <button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors"${attr("disabled", !store_get($$store_subs ??= {}, "$project", project), true)}>Save</button> <div class="w-px h-4 bg-border mx-1"></div> <button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors disabled:opacity-40 disabled:cursor-not-allowed"${attr("disabled", !store_get($$store_subs ??= {}, "$canUndo", canUndo), true)} title="Undo (Ctrl+Z)">Undo</button> <button class="px-3 py-1 text-xs bg-surface hover:bg-surface-hover text-text-primary rounded transition-colors disabled:opacity-40 disabled:cursor-not-allowed"${attr("disabled", !store_get($$store_subs ??= {}, "$canRedo", canRedo), true)} title="Redo (Ctrl+Y)">Redo</button></div> <div class="flex items-center gap-3 text-xs text-text-secondary">`);
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="font-medium text-text-primary">${escape_html(store_get($$store_subs ??= {}, "$project", project).project.name)}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (store_get($$store_subs ??= {}, "$isDirty", isDirty)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="text-warning">Modified</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (store_get($$store_subs ??= {}, "$projectPath", projectPath)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="truncate max-w-64"${attr("title", store_get($$store_subs ??= {}, "$projectPath", projectPath))}>${escape_html(store_get($$store_subs ??= {}, "$projectPath", projectPath))}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div></header>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function StatusBar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    const partCount = store_get($$store_subs ??= {}, "$cachedParts", cachedParts).length;
    const errorCount = store_get($$store_subs ??= {}, "$validationResult", validationResult)?.errors?.length ?? 0;
    const warningCount = store_get($$store_subs ??= {}, "$validationResult", validationResult)?.warnings?.length ?? 0;
    $$renderer2.push(`<footer class="flex items-center justify-between px-4 py-1 bg-bg-secondary border-t border-border text-xs text-text-secondary"><div class="flex items-center gap-4">`);
    if (store_get($$store_subs ??= {}, "$project", project)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span>Units: ${escape_html(store_get($$store_subs ??= {}, "$project", project).project.units)}</span> <span>Materials: ${escape_html(store_get($$store_subs ??= {}, "$project", project).materials.length + (store_get($$store_subs ??= {}, "$project", project).material ? 1 : 0))}</span> <span>Parts: ${escape_html(partCount)}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<span>No project loaded</span>`);
    }
    $$renderer2.push(`<!--]--></div> <div class="flex items-center gap-4">`);
    if (errorCount > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="text-error">${escape_html(errorCount)} error${escape_html(errorCount !== 1 ? "s" : "")}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (warningCount > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="text-warning">${escape_html(warningCount)} warning${escape_html(warningCount !== 1 ? "s" : "")}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (store_get($$store_subs ??= {}, "$isLoading", isLoading)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="animate-pulse">Working...</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div></footer>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function Toast($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    if (store_get($$store_subs ??= {}, "$toasts", toasts).length > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="fixed bottom-12 right-4 z-50 flex flex-col gap-2"><!--[-->`);
      const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$toasts", toasts));
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let toast = each_array[$$index];
        $$renderer2.push(`<div${attr_class("px-4 py-2 rounded shadow-lg text-sm text-white max-w-80 animate-fade-in", void 0, {
          "bg-blue-600": toast.type === "info",
          "bg-green-600": toast.type === "success",
          "bg-red-600": toast.type === "error",
          "bg-yellow-600": toast.type === "warning"
        })}>${escape_html(toast.message)}</div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    $$renderer2.push(`<div class="flex flex-col h-screen">`);
    TopBar($$renderer2);
    $$renderer2.push(`<!----> <div class="flex flex-1 overflow-hidden">`);
    Sidebar($$renderer2);
    $$renderer2.push(`<!----> <main class="flex-1 overflow-auto p-6">`);
    children($$renderer2);
    $$renderer2.push(`<!----></main></div> `);
    StatusBar($$renderer2);
    $$renderer2.push(`<!----> `);
    Toast($$renderer2);
    $$renderer2.push(`<!----></div>`);
  });
}
export {
  _layout as default
};
