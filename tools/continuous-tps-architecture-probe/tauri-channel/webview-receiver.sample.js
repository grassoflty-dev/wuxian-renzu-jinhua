// AG-16 evidence fixture only. This is the WebView-side shape emitted by
// tauri-2.11.5/src/ipc/channel.rs; it is not loaded by the campaign UI.
export function receiveContinuousSnapshot(event, state) {
  if (event?.end === true) {
    state.closed = true;
    state.lastIndex = event.index;
    return { kind: "close", index: event.index };
  }

  if (!Number.isInteger(event?.index) || event.index < state.lastIndex) {
    throw new Error("non-monotonic Tauri Channel index");
  }

  const message = typeof event.message === "string"
    ? JSON.parse(event.message)
    : event.message;
  state.lastIndex = event.index;
  state.frames.push(message);
  return { kind: "frame", index: event.index, message };
}
