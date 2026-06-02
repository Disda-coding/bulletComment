import { ref } from "vue";

const isPinned = ref(false);

export function useWindowManager() {
  async function togglePin() {
    isPinned.value = !isPinned.value;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const appWindow = getCurrentWindow();
    await appWindow.setIgnoreCursorEvents(isPinned.value);
  }

  async function createDanmakuWindow() {
    const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    const existing = await import("@tauri-apps/api/webviewWindow").then((m) =>
      m.getAllWebviewWindows()
    );
    const found = existing.find((w) => w.label === "danmaku");
    if (found) {
      await found.show();
      await found.setFocus();
      return;
    }

    const danmakuWindow = new WebviewWindow("danmaku", {
      url: "/danmaku",
      title: "弹幕显示",
      width: 1200,
      height: 400,
      x: 100,
      y: 50,
      transparent: true,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      resizable: true,
    });

    danmakuWindow.once("tauri://created", () => {});
    danmakuWindow.once("tauri://error", (e) => {
      console.error("Failed to create danmaku window:", e);
    });
  }

  async function closeDanmakuWindow() {
    const { getAllWebviewWindows } = await import("@tauri-apps/api/webviewWindow");
    const windows = await getAllWebviewWindows();
    const danmaku = windows.find((w) => w.label === "danmaku");
    if (danmaku) {
      await danmaku.close();
    }
  }

  return {
    isPinned,
    togglePin,
    createDanmakuWindow,
    closeDanmakuWindow,
  };
}
