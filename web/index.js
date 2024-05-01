const OFFIMG_PATH = "offimg";
const IMAGES_TAG = document.querySelector("[images]");
const INFO_TAG = document.querySelector("[info]");

fetch(`${OFFIMG_PATH}/info.json`)
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

    d.images.sort((a, b) => new Date(b.post.date) - new Date(a.post.date));
    IMAGES_TAG.innerHTML = "";
    for (let i of d.images) {
      let image = document.createElement("div");
      image.setAttribute("image", "");

      {
        let img = document.createElement("img");
        img.src = `${OFFIMG_PATH}/${i.uuid}.bmp`;
        img.alt = i.alt;
        image.appendChild(img);
      }

      {
        let foot = document.createElement("div");
        foot.setAttribute("foot", "");

        let info = document.createElement("div");
        info.innerHTML = `${new Date(i.post.date).toLocaleDateString(
          "en-US"
        )} &bull; `;

        let post = document.createElement("a");
        post.href = `https://forum.swissmicros.com/viewtopic.php?t=${i.post.post}`;
        post.innerHTML = `Post ${i.post.post}`;
        post.target = "_blank";
        post.rel = "noopener noreferrer";
        info.appendChild(post);

        if (i.alt) info.innerHTML += ` &bull; ${i.alt}`;

        let download = document.createElement("a");
        download.href = `${OFFIMG_PATH}/${i.uuid}.bmp`;
        download.innerHTML = `⇓`;
        if (i.alt) download.download = `${i.alt}${i.alt.endsWith(".bmp") ? "" : ".bmp"}`;
        else download.download = `${i.uuid}.bmp`;

        foot.appendChild(info);
        foot.appendChild(download);
        image.appendChild(foot);
      }

      IMAGES_TAG.appendChild(image);
    }
  });
