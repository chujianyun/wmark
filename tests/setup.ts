import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";
afterEach(cleanup);
class Observer {
  observe() {}
  unobserve() {}
  disconnect() {}
}
vi.stubGlobal("ResizeObserver", Observer);
window.HTMLElement.prototype.scrollIntoView = () => {};
Object.defineProperty(HTMLElement.prototype,'offsetHeight',{configurable:true,get:()=>400});
Object.defineProperty(HTMLElement.prototype,'offsetWidth',{configurable:true,get:()=>220});
