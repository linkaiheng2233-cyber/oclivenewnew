type EventCallback<T> = (event: { payload: T }) => void;

const listeners = new Map<string, Set<EventCallback<unknown>>>();

export async function listen<T>(
  event: string,
  handler: EventCallback<T>,
): Promise<() => void> {
  if (!listeners.has(event)) listeners.set(event, new Set());
  const set = listeners.get(event)!;
  set.add(handler as EventCallback<unknown>);
  return () => {
    set.delete(handler as EventCallback<unknown>);
  };
}

export async function emit<T>(event: string, payload: T): Promise<void> {
  const set = listeners.get(event);
  if (!set) return;
  for (const handler of set) {
    handler({ payload });
  }
}

export async function once<T>(
  event: string,
  handler: EventCallback<T>,
): Promise<() => void> {
  const unlisten = await listen<T>(event, (ev) => {
    void unlisten();
    handler(ev);
  });
  return unlisten;
}
