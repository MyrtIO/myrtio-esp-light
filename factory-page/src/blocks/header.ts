export class HeaderBlock {
  //   private block: HTMLElement;
  private loader: HTMLElement;
  private progressBar: HTMLElement;
  private saveButton: HTMLElement;

  constructor(element: HTMLElement, onSave: (e: Event) => void) {
    const $ = (selector: string): HTMLDivElement => {
      const node = element.querySelector<HTMLDivElement>(selector);
      if (!node) {
        throw new Error(
          `Element with selector ${selector} not found in header`
        );
      }
      return node;
    };
    // this.block = element;
    this.progressBar = $(".progress-bar");
    this.loader = $(".loader");
    this.saveButton = $("#button-save");
    this.saveButton.addEventListener("click", onSave);
  }

  public showLoader() {
    this.loader.classList.add("_visible");
  }

  public hideLoader() {
    this.loader.classList.remove("_visible");
  }

  public showSaveButton() {
    this.saveButton.classList.remove("_hidden");
  }

  public hideSaveButton() {
    this.saveButton.classList.add("_hidden");
  }

  public showProgressBar() {
    this.progressBar.classList.add("_visible");
  }

  public hideProgressBar() {
    this.progressBar.classList.remove("_visible");
  }

  public setProgress(progress: number) {
    this.progressBar.style.setProperty("--progress-bar-value", `${progress}%`);
  }
}
