import { describe, expect, it, vi } from "vitest";
import { executeAndClose } from "./action-lifecycle";

describe("Quick Panel external action lifecycle", () => {
  it("closes the panel after opening an external Windows destination", async () => {
    const calls: string[] = [];
    const open = vi.fn(async () => { calls.push("open"); });
    const close = vi.fn(async () => { calls.push("close"); });

    await expect(executeAndClose(open, close)).resolves.toBe("closed");
    expect(calls).toEqual(["open", "close"]);
    expect(close).toHaveBeenCalledOnce();
  });

  it("does not close the panel if the external action fails", async () => {
    const close = vi.fn(async () => {});
    const result = await executeAndClose(
      async () => { throw new Error("Windows launch failed"); },
      close,
    );

    expect(result).toBe("open-failed");
    expect(close).not.toHaveBeenCalled();
  });

  it("distinguishes a failed panel close from a failed external launch", async () => {
    const open = vi.fn(async () => {});
    const result = await executeAndClose(
      open,
      async () => { throw new Error("Window hide failed"); },
    );

    expect(result).toBe("close-failed");
    expect(open).toHaveBeenCalledOnce();
  });
});
