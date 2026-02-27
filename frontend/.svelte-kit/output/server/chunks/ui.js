import { w as writable } from "./index.js";
const selectedCabinetIndex = writable(null);
const sidebarCollapsed = writable(false);
const selectedMaterialIndex = writable(0);
const selectedSheetIndex = writable(0);
export {
  selectedCabinetIndex as a,
  selectedMaterialIndex as b,
  selectedSheetIndex as c,
  sidebarCollapsed as s
};
