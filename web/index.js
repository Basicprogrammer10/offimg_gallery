const IMAGES_TAG = document.querySelector("[images]");
const INFO_TAG = document.querySelector("[info]");

fetch("out/info.json")
  .then((d) => d.json())
  .then((d) => {
    for (let i of [
      `Last updated: ${new Date(d.date).toLocaleString("en-US")}`,
      `Images: ${d.images.length}`,
    ]) {
      let item = document.createElement("li");
      item.innerHTML = i;
      INFO_TAG.appendChild(item);
    }

    d.images.sort((a, b) => new Date(b.date) - new Date(a.date));
    IMAGES_TAG.innerHTML = "";
    for (let i of d.images) {
      let image = document.createElement("div");
      image.setAttribute("image", "");

      {
        let img = document.createElement("img");
        img.src = `out/${i.id}.bmp`;
        img.alt = i.alt;
        image.appendChild(img);
      }

      {
        let foot = document.createElement("div");
        foot.setAttribute("foot", "");

        let info = document.createElement("div");
        info.innerHTML = `${new Date(i.date).toLocaleDateString(
          "en-US"
        )} &bull; `;

        let post = document.createElement("a");
        post.href = `https://forum.swissmicros.com/viewtopic.php?t=${i.post}`;
        post.innerHTML = `Post ${i.post}`;
        post.target = "_blank";
        post.rel = "noopener noreferrer";
        info.appendChild(post);

        info.innerHTML += ` &bull; ${i.alt}`;

        let download = document.createElement("a");
        download.href = `out/${i.id}.bmp`;
        download.innerHTML = `⇓`;
        download.download = `${i.alt}${i.alt.endsWith(".bmp") ? "" : ".bmp"}`;

        foot.appendChild(info);
        foot.appendChild(download);
        image.appendChild(foot);
      }

      IMAGES_TAG.appendChild(image);
    }
  });
