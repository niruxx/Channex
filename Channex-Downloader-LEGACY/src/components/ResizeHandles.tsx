import type { MouseEvent } from "react";
import { appWindow } from "../lib/appWindow";

const EDGE = 6;
const CORNER = 12;

function handle(direction: Parameters<typeof appWindow.startResizeDragging>[0]) {
  return (e: MouseEvent) => {
    if (e.buttons !== 1) return;
    e.preventDefault();
    void appWindow.startResizeDragging(direction);
  };
}

export function ResizeHandles() {
  return (
    <div className="pointer-events-none fixed inset-0 z-50">
      <div
        onMouseDown={handle("North")}
        className="pointer-events-auto absolute inset-x-0 top-0 cursor-ns-resize"
        style={{ height: EDGE }}
      />
      <div
        onMouseDown={handle("South")}
        className="pointer-events-auto absolute inset-x-0 bottom-0 cursor-ns-resize"
        style={{ height: EDGE }}
      />
      <div
        onMouseDown={handle("West")}
        className="pointer-events-auto absolute inset-y-0 left-0 cursor-ew-resize"
        style={{ width: EDGE }}
      />
      <div
        onMouseDown={handle("East")}
        className="pointer-events-auto absolute inset-y-0 right-0 cursor-ew-resize"
        style={{ width: EDGE }}
      />

      <div
        onMouseDown={handle("NorthWest")}
        className="pointer-events-auto absolute left-0 top-0 cursor-nwse-resize"
        style={{ width: CORNER, height: CORNER }}
      />
      <div
        onMouseDown={handle("NorthEast")}
        className="pointer-events-auto absolute right-0 top-0 cursor-nesw-resize"
        style={{ width: CORNER, height: CORNER }}
      />
      <div
        onMouseDown={handle("SouthWest")}
        className="pointer-events-auto absolute bottom-0 left-0 cursor-nesw-resize"
        style={{ width: CORNER, height: CORNER }}
      />
      <div
        onMouseDown={handle("SouthEast")}
        className="pointer-events-auto absolute bottom-0 right-0 cursor-nwse-resize"
        style={{ width: CORNER, height: CORNER }}
      />
    </div>
  );
}
