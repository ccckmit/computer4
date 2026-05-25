let ws = null;
let reconnectTimer = null;

function connect() {
  const proto = location.protocol === "https:" ? "wss:" : "ws:";
  ws = new WebSocket(`${proto}//${location.host}`);

  ws.onopen = () => {
    document.getElementById("ws-status").textContent = "已連線";
    document.getElementById("ws-status").className = "connected";
  };

  ws.onmessage = (e) => {
    try {
      const msg = JSON.parse(e.data);
      if (msg.type === "leaderboard") renderLeaderboard(msg.scores);
    } catch (_) {}
  };

  ws.onclose = () => {
    document.getElementById("ws-status").textContent = "連線中斷，重新連線中...";
    document.getElementById("ws-status").className = "disconnected";
    reconnectTimer = setTimeout(connect, 2000);
  };

  ws.onerror = () => ws?.close();
}

function sendScore(result) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({
      type: "score",
      player_score: result.score[0],
      ai_score: result.score[1],
      winner: result.winner,
      rally: result.rally,
    }));
  }
}

function renderLeaderboard(scores) {
  const el = document.getElementById("leaderboard");
  if (!scores || scores.length === 0) {
    el.innerHTML = "<div class='empty'>尚無紀錄</div>";
    return;
  }
  let html = "";
  for (const s of scores) {
    html += `<div class="entry">
      <span class="winner ${s.winner === '玩家' ? 'win' : 'lose'}">${s.winner}</span>
      <span class="result">${s.player_score} - ${s.ai_score}</span>
      <span class="rally">rally ${s.rally}</span>
    </div>`;
  }
  el.innerHTML = html;
}

const canvas = document.getElementById("pong-canvas");
const engine = new game4.Engine(canvas);
const pong = new game4.Pong();

engine.onEnd = (result) => sendScore(result);

window.addEventListener("keydown", (e) => {
  if ((e.key === "r" || e.key === "R") && pong.done) pong.restart();
});

connect();
engine.start(pong);
