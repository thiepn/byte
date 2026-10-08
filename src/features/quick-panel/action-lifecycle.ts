/**
 * Native actions often transfer focus away from Byte. Close the panel
 * explicitly after the external destination opens so it cannot remain visible
 * with its refresh loop stopped (or intercept the new application's clicks).
 */
export type ActionCloseResult = "closed" | "open-failed" | "close-failed";

export async function executeAndClose(
  openAction: () => Promise<void>,
  closePanel: () => Promise<void>,
): Promise<ActionCloseResult> {
  try {
    await openAction();
  } catch {
    return "open-failed";
  }

  try {
    await closePanel();
    return "closed";
  } catch {
    return "close-failed";
  }
}
