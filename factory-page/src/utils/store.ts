type Listener<T> = (value: T, prev: T) => void;

/**
 * A simple atomic store implementation.
 */
export class Store<T> {
  private value: T;
  private listeners = new Set<Listener<T>>();

  constructor(initial: T) {
    this.value = initial;
  }

  /**
   * Get the current value of the atom.
   */
  get(): T {
    return this.value;
  }

  /**
   * Set the value of the atom.
   * @param next - The new value or a function to update the value.
   */
  set(next: T | ((prev: T) => T)) {
    const prev = this.value;
    const value =
      typeof next === "function" ? (next as (p: T) => T)(prev) : next;

    if (Object.is(value, prev)) return;

    this.value = value;
    this.listeners.forEach((l) => l(value, prev));
  }

  /**
   * Subscribe to changes to the atom.
   * @param listener - The listener to call when the value changes.
   * @returns A function to unsubscribe from the atom.
   */
  subscribe(listener: Listener<T>, immediate = false): () => void {
    if (immediate) {
      listener(this.value, this.value);
    }
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}
