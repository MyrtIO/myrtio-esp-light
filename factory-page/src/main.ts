import "./style.css";

const header = document.getElementById("header")!;
window.addEventListener("scroll", () => {
  header.classList.toggle("scrolled", window.scrollY > 0);
});

const configForm = document.getElementById("configForm")!;
const statusOverlay = document.getElementById("statusOverlay")!;
const statusText = document.getElementById("statusText")!;
const saveBtn = document.getElementById("saveBtn")!;
const otaFile = document.getElementById("otaFile")!;
const uploadTrigger = document.getElementById("uploadTrigger")!;
const progressBar = document.getElementById("progressBar")!;
const progressContainer = document.getElementById("progressContainer")!;

const showStatus = (text: string, type = "info", duration = 0) => {
  statusText.textContent = text;
  statusOverlay.style.display = "flex";
  const spinner: HTMLElement = statusOverlay.querySelector(".spinner")!;
  spinner.style.display =
    type === "loading" ? "block" : "none";
  statusOverlay.style.color =
    type === "error"
      ? "var(--danger-color)"
      : type === "success"
      ? "var(--success-color)"
      : "var(--text-primary)";
  if (duration > 0)
    setTimeout(() => (statusOverlay.style.display = "none"), duration);
};

fetch("/config")
  .then((r) => r.json())
  .then((c) => {
    for (let [k, v] of Object.entries(c)) {
      const el = document.getElementById(k)! as HTMLInputElement;
      let value = v as string;
      if (el) el.value = k === "color_correction" ? value.replace("#", "") : value;
    }
  })
  .catch(() => {});

configForm.addEventListener("input", (e) => {
  if ((e.target as HTMLInputElement).type !== "password") {
    saveBtn.style.display = "block";
  }
});

configForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  showStatus("Сохранение...", "loading");
  try {
    const form = new FormData(configForm as HTMLFormElement);
    const params = new URLSearchParams(form as any).toString();
    const r = await fetch("/config", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: params,
    });
    const t = await r.text();
    if (r.ok) {
      showStatus("Сохранено", "success", 2000);
      saveBtn.style.display = "none";
    } else {
      showStatus(t || "Ошибка", "error", 3000);
    }
  } catch (err) {
    showStatus("Ошибка сети", "error", 3000);
  }
});

uploadTrigger.addEventListener("click", (e) => {
  e.preventDefault();
  otaFile.click();
});

otaFile.addEventListener("change", async () => {
  const f = (otaFile as HTMLInputElement).files?.[0];
  if (!f) return;

  if (!confirm(`Обновить прошивку файлом ${f.name}?`)) {
    (otaFile as HTMLInputElement).value = "";
    return;
  }

  showStatus("Загрузка...", "loading");
  progressContainer.style.display = "block";
  progressBar.style.width = "0%";

  try {
    const xhr = new XMLHttpRequest();
    xhr.open("POST", "/ota");
    xhr.setRequestHeader("Content-Type", "application/octet-stream");
    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable)
        progressBar.style.width = (e.loaded / e.total) * 100 + "%";
    };
    xhr.onload = () => {
      if (xhr.status === 200) {
        showStatus("Готово! Перезагрузка...", "success");
        setTimeout(() => location.reload(), 3000);
      } else {
        showStatus(xhr.responseText || "Ошибка при загрузке", "error", 5000);
        progressContainer.style.display = "none";
      }
    };
    xhr.onerror = () => {
      showStatus("Ошибка загрузки", "error", 5000);
      progressContainer.style.display = "none";
    };
    xhr.send(f);
  } catch (err) {
    showStatus("Ошибка сети", "error", 3000);
    progressContainer.style.display = "none";
  }
});
