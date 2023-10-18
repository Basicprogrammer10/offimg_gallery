const IMAGES_TAG = document.querySelector("[images]");

fetch('out/info.json').then(d => d.json()).then(d => {
    console.log(d);
    for (let i of d.images) {
        let img = document.createElement("img");
        img.src = `out/${i.filename}`;
        img.alt = i.alt;
        IMAGES_TAG.appendChild(img);
    }
});