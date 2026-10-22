"use strict";
const grid = document.querySelector("#specimens");
const status = document.querySelector("#status");
const zoom = document.querySelector("#zoom");
let specimens = [];
function display() {
  const query = new URLSearchParams(location.search);
  const chosen = query.get("id");
  const visible = specimens.filter(item => item.zoom === Number(zoom.value) && (!chosen || item.id === chosen));
  grid.replaceChildren();
  for (const item of visible) {
    const figure = document.createElement("figure");
    figure.id = item.id;
    const img = document.createElement("img");
    img.src = item.file; img.alt = `Specimen ${item.id.slice(-2)}`;
    img.width = item.width; img.height = item.height;
    img.addEventListener("error", () => { status.textContent = `Load failed: ${item.id}`; });
    const caption = document.createElement("figcaption");
    caption.textContent = `${item.id.slice(-2)} · ${item.zoom}×`;
    figure.append(img, caption); grid.append(figure);
  }
  status.textContent = `${visible.length} specimens · native source dimensions`;
  window.labReady = true;
}
zoom.addEventListener("change", display);
document.querySelector("#ground").addEventListener("change", event => {
  document.body.classList.toggle("forest", event.target.value === "forest");
});
document.querySelector("#grid").addEventListener("change", event => {
  document.body.classList.toggle("clear-grid", !event.target.checked);
});
fetch("manifest.json").then(response => {
  if (!response.ok) throw new Error(`Manifest HTTP ${response.status}`);
  return response.json();
}).then(data => {
  specimens = data.specimens;
  zoom.value = new URLSearchParams(location.search).get("zoom") === "2" ? "2" : "1";
  display();
}).catch(error => { status.textContent = `Viewer unavailable: ${error.message}`; });
