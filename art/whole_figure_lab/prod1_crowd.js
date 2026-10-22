"use strict";
(async () => {
  const response = await fetch("crowd_manifest.json");
  if (!response.ok) throw new Error(`Manifest HTTP ${response.status}`);
  const data = await response.json();
  const canvas = document.createElement("canvas");
  canvas.width = 2240; canvas.height = 1120;
  canvas.setAttribute("aria-label", "200 whole-figure sprites at native scale");
  document.body.replaceChildren(canvas);
  document.body.style.cssText = "margin:0;padding:0;overflow:hidden";
  const context = canvas.getContext("2d", {alpha: false});
  context.imageSmoothingEnabled = false;
  async function decode(source) {
    const image = new Image(); image.src = source;
    await image.decode(); return image;
  }
  const ground = await decode("meadow.png");
  const sprites = await Promise.all(data.sprites.map(async row => ({
    activity: row.activity,
    frames: await Promise.all(row.files.map(file => decode(`textures/${file}`)))
  })));
  const plan = {idle:[0,0],walk:[0,1],work:[0,2],carry:[3,3],sleep:[4,4]};
  const intervals = [];
  let start, previous, draws = 0;
  function tick(now) {
    if (start === undefined) start = now;
    if (previous !== undefined) intervals.push(now - previous);
    previous = now;
    for (let y = 0; y < canvas.height; y += ground.height) {
      for (let x = 0; x < canvas.width; x += ground.width) context.drawImage(ground,x,y);
    }
    const phase = Math.floor((now - start) / 125) % 2;
    for (let index = 0; index < 200; index++) {
      const sprite = sprites[index % sprites.length];
      const image = sprite.frames[plan[sprite.activity][phase]];
      const bob = phase && (sprite.activity === "walk" || sprite.activity === "carry") ? -1 : 0;
      const x = (index % 20) * 112 + (112-image.width)/2;
      const y = Math.floor(index / 20) * 112 + 104-(image.width === 112 ? 72 : 104)+bob;
      const cx = (index % 20) * 112 + 56;
      const cy = Math.floor(index / 20) * 112 + 104;
      context.fillStyle = "rgba(24,17,10,0.25)";
      context.beginPath(); context.ellipse(cx,cy,14,4,0,0,Math.PI*2); context.fill();
      context.drawImage(image,x,y); draws++;
    }
    window.crowdReady = true;
    if (now - start >= 60000) {
      window.crowdResult = {dataset_status:data.dataset_status,count:200,visible_count:200,canvas:[2240,1120],elapsed_ms:now-start,
        frame_intervals_ms:intervals,rendered_frames:intervals.length+1,draw_calls:draws,
        fps:intervals.length*1000/(now-start),surface:"Chromium Canvas2D whole-sprite rendering; no simulation"};
    } else requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);
})().catch(error => { window.crowdError = String(error); document.body.textContent=String(error); });
