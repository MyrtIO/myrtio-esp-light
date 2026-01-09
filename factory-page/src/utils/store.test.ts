import { describe, expect, it, vi } from "vitest";

import { Store } from "./store";

describe("Store", () => {
  it("returns the initial value via get()", () => {
    const store = new Store(42);

    expect(store.get()).toBe(42);
  });

  it("updates value and notifies listeners when set() is called", () => {
    const store = new Store("old");
    const listener = vi.fn(() => {});
    store.subscribe(listener);

    store.set("new");

    expect(store.get()).toBe("new");
    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener).toHaveBeenCalledWith("new", "old");
  });

  it("supports updater functions in set()", () => {
    const store = new Store({ count: 0 });

    store.set((prev) => ({ count: prev.count + 1 }));

    expect(store.get()).toEqual({ count: 1 });
  });

  it("does not notify listeners when the value stays identical", () => {
    const store = new Store(NaN);
    const listener = vi.fn(() => {});
    store.subscribe(listener);

    store.set(NaN);

    expect(listener).not.toHaveBeenCalled();
  });

  it("allows unsubscribing listeners", () => {
    const store = new Store(0);
    const listener = vi.fn(() => {});
    const unsubscribe = store.subscribe(listener);

    store.set(1);
    unsubscribe();
    store.set(2);

    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener).toHaveBeenLastCalledWith(1, 0);
  });
});
