const CDN_URL = "/tracks";

const listEl = document.getElementById("song-list");
const playPauseBtn = document.getElementById("play-pause-btn");
const prevBtn = document.getElementById("back-btn");
const nextBtn = document.getElementById("next-btn");

const audio = new Audio();
audio.volume = 1;

let currentIndex = null;
let trackList = [];

function urlDecode(str) {
  try {
    return decodeURIComponent(str).split(".mp3")[0];
  } catch {
    return str;
  }
}

async function fetchTracks() {
  const res = await fetch(CDN_URL);
  trackList = await res.json();
  renderList();
}

function renderList() {
  listEl.innerHTML = "";
  trackList.forEach((track, index) => {
    const li = document.createElement("li");
    li.className = "song-list-item";
    li.id = "track-" + index;

    const indexSpan = document.createElement("span");
    indexSpan.className = "song-list-index";
    indexSpan.textContent = index + 1;

    const trackNameSpan = document.createElement("span");
    trackNameSpan.textContent = urlDecode(track);

    li.appendChild(indexSpan);
    li.appendChild(trackNameSpan);

    li.onclick = () => playTrack(index);

    listEl.appendChild(li);
  });
}

function playTrack(index) {
  if (currentIndex !== null) {
    document
      .getElementById("track-" + currentIndex)
      .classList.remove("active-song");
  }

  currentIndex = index;
  audio.src = `${CDN_URL}/${trackList[index]}`;
  audio.play();

  document.getElementById("track-" + index).classList.add("active-song");
  playPauseBtn.innerHTML = "❚❚";
}

playPauseBtn.onclick = () => {
  if (audio.paused) {
    if (currentIndex === null && trackList.length > 0) {
      playTrack(0);
    } else {
      audio.play();
      playPauseBtn.innerHTML = "❚❚";
    }
  } else {
    audio.pause();
    playPauseBtn.innerHTML = "▶";
  }
};

prevBtn.onclick = () => {
  if (currentIndex === null) return;
  let prevIndex = currentIndex - 1;
  if (prevIndex < 0) prevIndex = trackList.length - 1;
  playTrack(prevIndex);
};

nextBtn.onclick = () => {
  if (currentIndex === null) return;
  let nextIndex = currentIndex + 1;
  if (nextIndex >= trackList.length) nextIndex = 0;
  playTrack(nextIndex);
};

audio.onended = () => {
  nextBtn.onclick();
};

fetchTracks();
