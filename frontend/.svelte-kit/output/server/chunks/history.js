import { d as derived, w as writable, g as get } from "./index.js";
import { p as project } from "./project.js";
const MAX_SNAPSHOTS = 50;
const undoStack = writable([]);
const redoStack = writable([]);
const canUndo = derived(undoStack, ($stack) => $stack.length > 0);
const canRedo = derived(redoStack, ($stack) => $stack.length > 0);
function pushSnapshot() {
  const current = get(project);
  if (!current) return;
  const snapshot = JSON.stringify(current);
  undoStack.update((stack) => {
    const next = [...stack, snapshot];
    if (next.length > MAX_SNAPSHOTS) next.shift();
    return next;
  });
  redoStack.set([]);
}
export {
  canRedo as a,
  canUndo as c,
  pushSnapshot as p
};
