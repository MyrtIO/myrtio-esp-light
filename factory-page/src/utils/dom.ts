type FormSeeker = <T extends HTMLElement = HTMLInputElement>(name: string) => T;

export function createElementSeeker(
  element: HTMLElement
): FormSeeker {
  return <T extends HTMLElement = HTMLInputElement>(name: string) => {
    const input = element.querySelector(`[name="${name}"]`) as T;
    if (!input) {
      throw new Error(`Element with name "${name}" not found`);
    }
    return input;
  };
}
