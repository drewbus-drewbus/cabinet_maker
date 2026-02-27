import { d as derived, w as writable } from "./index.js";
const project = writable(null);
const cachedParts = writable([]);
const nestingResults = writable([]);
const validationResult = writable(null);
const cutlistRows = writable([]);
const isDirty = writable(false);
const projectPath = writable(null);
const isLoading = writable(false);
const toasts = writable([]);
let toastId = 0;
function showToast(message, type = "info") {
  const id = ++toastId;
  toasts.update((t) => [...t, { id, message, type }]);
  setTimeout(() => {
    toasts.update((t) => t.filter((toast) => toast.id !== id));
  }, 4e3);
}
const cabinetCount = derived(project, ($project) => {
  if (!$project) return 0;
  let count = $project.cabinets.length;
  if ($project.cabinet) count += 1;
  return count;
});
const totalPartCount = derived(cachedParts, ($parts) => $parts.length);
derived(
  validationResult,
  ($result) => $result !== null && $result.errors.length > 0
);
export {
  projectPath as a,
  cachedParts as b,
  cabinetCount as c,
  isLoading as d,
  toasts as e,
  cutlistRows as f,
  isDirty as i,
  nestingResults as n,
  project as p,
  showToast as s,
  totalPartCount as t,
  validationResult as v
};
