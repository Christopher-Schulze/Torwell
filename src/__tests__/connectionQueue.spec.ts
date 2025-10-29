import { beforeEach, describe, expect, it } from "vitest";
import { get } from "svelte/store";
import { connectionQueue } from "../lib/utils/actionQueue";

beforeEach(() => {
  connectionQueue.reset();
});

describe("connectionQueue", () => {
  it("runs actions sequentially", async () => {
    const order: string[] = [];
    const first = connectionQueue.run("connect", async () => {
      order.push("connect:start");
      await Promise.resolve();
      order.push("connect:end");
    });
    const second = connectionQueue.run("disconnect", async () => {
      order.push("disconnect");
    });

    const [firstResult, secondResult] = await Promise.all([first, second]);
    expect(firstResult.status).toBe("completed");
    expect(secondResult.status).toBe("completed");
    expect(order).toEqual(["connect:start", "connect:end", "disconnect"]);
  });

  it("skips duplicate keys while active", async () => {
    const first = connectionQueue.run("connect", async () => {
      await Promise.resolve();
    });
    const duplicate = await connectionQueue.run("connect", async () => {
      throw new Error("should not run");
    });

    expect(duplicate.status).toBe("skipped");
    await first;

    const after = await connectionQueue.run("connect", async () => {});
    expect(after.status).toBe("completed");
  });

  it("records errors and exposes them via the store", async () => {
    await expect(
      connectionQueue.run("connect", async () => {
        throw new Error("boom");
      }),
    ).rejects.toThrow("boom");

    const state = get(connectionQueue);
    expect(state.lastError?.key).toBe("connect");
    expect(state.lastError?.message).toContain("boom");
  });
});
