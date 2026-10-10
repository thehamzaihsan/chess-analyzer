type Child = string | Node | null | undefined | false;

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Record<string, string | number | boolean | undefined | ((e: Event) => void)> = {},
  children: Child[] = [],
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value === undefined || value === false) continue;
    if (key.startsWith('on') && typeof value === 'function') {
      node.addEventListener(key.slice(2).toLowerCase(), value as EventListener);
    } else if (key === 'class') {
      node.className = String(value);
    } else if (value === true) {
      node.setAttribute(key, '');
    } else {
      node.setAttribute(key, String(value));
    }
  }
  for (const child of children) {
    if (child == null || child === false) continue;
    node.append(typeof child === 'string' ? document.createTextNode(child) : child);
  }
  return node;
}

export function text(value: string): Text {
  return document.createTextNode(value);
}

export function clear(node: Element): void {
  node.innerHTML = '';
}

/** A native <details>/<summary> accordion: keyboard-accessible, no JS needed to toggle. */
export function accordion(summaryText: string, children: Child[], open = false): HTMLDetailsElement {
  return el('details', { class: 'accordion', open }, [
    el('summary', {}, [summaryText]),
    el('div', { class: 'accordion-body' }, children),
  ]);
}

export function escapeHtml(value: string): string {
  const div = document.createElement('div');
  div.textContent = value;
  return div.innerHTML;
}
