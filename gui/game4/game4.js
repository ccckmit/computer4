const game4 = (() => {
  // ── Input ──────────────────────────────────────
  class Input {
    constructor() {
      this.up = false;
      this.down = false;
      this.left = false;
      this.right = false;
      this.action = false;
    }
    static none() { return new Input(); }
    static up() { const i = new Input(); i.up = true; return i; }
    static down() { const i = new Input(); i.down = true; return i; }
  }

  // ── Game (abstract base) ─────────────────────
  class Game {
    init() {}
    step(input) { return { done: false, winner: null, score: [0, 0], info: {} }; }
    render(ctx, w, h) {}
    title() { return ""; }
  }

  // ── Engine ────────────────────────────────────
  class Engine {
    constructor(canvasEl, opts = {}) {
      this.canvas = typeof canvasEl === "string" ? document.getElementById(canvasEl) : canvasEl;
      this.ctx = this.canvas.getContext("2d");
      this.fps = opts.fps || 60;
      this.keys = {};
      this.input = new Input();
      this.game = null;
      this.running = false;
      this._lastTime = 0;
      this._animId = null;
      this.onEnd = null;
      this._setupInput();
    }

    start(game) {
      this.game = game;
      game.init();
      this.running = true;
      this._lastTime = performance.now();
      this._loop(this._lastTime);
    }

    stop() {
      this.running = false;
      if (this._animId) { cancelAnimationFrame(this._animId); this._animId = null; }
    }

    _setupInput() {
      const mapKey = (e) => {
        this.keys[e.key] = e.type === "keydown";
        if (["ArrowUp","ArrowDown","ArrowLeft","ArrowRight"," "].includes(e.key)) e.preventDefault();
      };
      window.addEventListener("keydown", mapKey);
      window.addEventListener("keyup", mapKey);
    }

    _pollInput() {
      this.input.up = !!this.keys["ArrowUp"] || !!this.keys["w"] || !!this.keys["W"];
      this.input.down = !!this.keys["ArrowDown"] || !!this.keys["s"] || !!this.keys["S"];
      this.input.left = !!this.keys["ArrowLeft"] || !!this.keys["a"] || !!this.keys["A"];
      this.input.right = !!this.keys["ArrowRight"] || !!this.keys["d"] || !!this.keys["D"];
      this.input.action = !!this.keys[" "];
    }

    _loop(now) {
      if (!this.running) return;
      this._animId = requestAnimationFrame((t) => this._loop(t));
      const elapsed = now - this._lastTime;
      if (elapsed < 1000 / this.fps) return;
      this._lastTime = now - (elapsed % (1000 / this.fps));
      this._pollInput();
      if (this.game) {
        const result = this.game.step(this.input);
        this.game.render(this.ctx, this.canvas.width, this.canvas.height);
        if (result && result.done && this.onEnd) this.onEnd(result);
      }
    }
  }

  // ── Drawing utilities ─────────────────────────
  const draw = {
    clear(ctx, w, h) { ctx.clearRect(0, 0, w, h); },
    rect(ctx, x, y, w, h, color) { ctx.fillStyle = color; ctx.fillRect(x, y, w, h); },
    circle(ctx, x, y, r, color) { ctx.fillStyle = color; ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2); ctx.fill(); },
    text(ctx, text, x, y, color, size = "16px", font = "monospace") {
      ctx.fillStyle = color;
      ctx.font = `${size} ${font}`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(text, x, y);
    },
  };

  // ── Vec2 ──────────────────────────────────────
  class Vec2 {
    constructor(x = 0, y = 0) { this.x = x; this.y = y; }
    add(v) { return new Vec2(this.x + v.x, this.y + v.y); }
    sub(v) { return new Vec2(this.x - v.x, this.y - v.y); }
    scale(s) { return new Vec2(this.x * s, this.y * s); }
    len() { return Math.sqrt(this.x * this.x + this.y * this.y); }
    clone() { return new Vec2(this.x, this.y); }
  }

  // ── Pong game ─────────────────────────────────
  const W = 800, H = 500;
  const PADDLE_W = 10, PADDLE_H = 50;
  const PADDLE_X = 20, AI_X = W - 20 - PADDLE_W;
  const BALL_R = 6, BALL_SPEED = 6, PADDLE_SPEED = 8;
  const WIN_SCORE = 5;

  class Pong extends Game {
    init() {
      this.ball_x = W / 2;
      this.ball_y = H / 2;
      this.ball_vx = (Math.random() < 0.5 ? 1 : -1) * BALL_SPEED * 0.8;
      this.ball_vy = BALL_SPEED * 0.2;
      this.player_y = H / 2;
      this.ai_y = H / 2;
      this.player_score = 0;
      this.ai_score = 0;
      this.done = false;
      this.winner = "";
      this.rally = 0;
      this.angle_history = [0.5, 0.3, -0.3, -0.5, 0.0, 0.6, -0.6, 0.2, -0.2];
    }

    step(input) {
      if (this.done) return { done: true, winner: this.winner, score: [this.player_score, this.ai_score], rally: this.rally };

      if (input.up) this.player_y = Math.max(this.player_y - PADDLE_SPEED, PADDLE_H / 2);
      if (input.down) this.player_y = Math.min(this.player_y + PADDLE_SPEED, H - PADDLE_H / 2);

      this._aiStep();
      this.ball_x += this.ball_vx;
      this.ball_y += this.ball_vy;

      if (this.ball_y - BALL_R <= 0) { this.ball_y = BALL_R; this.ball_vy = Math.abs(this.ball_vy); }
      if (this.ball_y + BALL_R >= H) { this.ball_y = H - BALL_R; this.ball_vy = -Math.abs(this.ball_vy); }

      // player paddle hit
      if (this.ball_vx < 0 &&
          this.ball_x - BALL_R <= PADDLE_X + PADDLE_W &&
          this.ball_x + BALL_R >= PADDLE_X &&
          Math.abs(this.ball_y - this.player_y) <= PADDLE_H / 2 + BALL_R) {
        this.ball_vx = -this.ball_vx;
        this.ball_x = PADDLE_X + PADDLE_W + BALL_R;
        this.ball_vy += (Math.random() - 0.5) * 1.6;
        this.ball_vy = Math.max(-BALL_SPEED, Math.min(BALL_SPEED, this.ball_vy));
        this.rally++;
      }

      // AI paddle hit
      if (this.ball_vx > 0 &&
          this.ball_x + BALL_R >= AI_X &&
          this.ball_x - BALL_R <= AI_X + PADDLE_W &&
          Math.abs(this.ball_y - this.ai_y) <= PADDLE_H / 2 + BALL_R) {
        this.ball_vx = -this.ball_vx;
        this.ball_x = AI_X - BALL_R;
        this.ball_vy += (Math.random() - 0.5) * 1.6;
        this.ball_vy = Math.max(-BALL_SPEED, Math.min(BALL_SPEED, this.ball_vy));
        this.rally++;
      }

      let result = null;
      if (this.ball_x + BALL_R < 0) {
        this.ai_score++;
        if (this.ai_score >= WIN_SCORE) {
          this.done = true;
          this.winner = "AI";
          result = { done: true, winner: "AI", score: [this.player_score, this.ai_score], rally: this.rally };
        } else this._reset();
      }
      if (this.ball_x - BALL_R > W) {
        this.player_score++;
        if (this.player_score >= WIN_SCORE) {
          this.done = true;
          this.winner = "玩家";
          result = { done: true, winner: "玩家", score: [this.player_score, this.ai_score], rally: this.rally };
        } else this._reset();
      }

      return result || { done: false, winner: null, score: [this.player_score, this.ai_score], rally: this.rally };
    }

    restart() {
      this.done = false;
      this.winner = "";
      this.player_score = 0;
      this.ai_score = 0;
      this.rally = 0;
      this._reset();
    }

    _reset() {
      const idx = (this.player_score + this.ai_score) % this.angle_history.length;
      const angle = this.angle_history[idx];
      const dir = (this.player_score + this.ai_score) % 2 === 0 ? 1 : -1;
      this.ball_x = W / 2;
      this.ball_y = H / 2;
      this.ball_vx = dir * BALL_SPEED * Math.cos(angle);
      this.ball_vy = BALL_SPEED * Math.sin(angle);
      this.player_y = H / 2;
      this.ai_y = H / 2;
    }

    _aiStep() {
      const target = this.ball_y + Math.max(-60, Math.min(60, this.ball_vy * 8));
      const diff = target - this.ai_y;
      if (Math.abs(diff) > PADDLE_H * 0.15) {
        const step = PADDLE_SPEED * 0.10;
        this.ai_y += diff > 0 ? Math.min(step, diff) : Math.max(-step, diff);
        this.ai_y = Math.max(PADDLE_H / 2, Math.min(H - PADDLE_H / 2, this.ai_y));
      }
    }

    render(ctx, w, h) {
      ctx.fillStyle = "#0a0a1a";
      ctx.fillRect(0, 0, W, H);

      ctx.strokeStyle = "#1a1a3a";
      ctx.lineWidth = 2;
      ctx.strokeRect(2, 2, W - 4, H - 4);

      ctx.setLineDash([10, 10]);
      ctx.strokeStyle = "#2a2a5a";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(W / 2, 0);
      ctx.lineTo(W / 2, H);
      ctx.stroke();
      ctx.setLineDash([]);

      const grad = ctx.createLinearGradient(20, 0, 20 + PADDLE_W, 0);
      grad.addColorStop(0, "#00E5FF");
      grad.addColorStop(1, "#00B8D4");
      ctx.fillStyle = grad;
      ctx.shadowColor = "#00E5FF";
      ctx.shadowBlur = 10;
      ctx.fillRect(PADDLE_X, this.player_y - PADDLE_H / 2, PADDLE_W, PADDLE_H);
      ctx.shadowBlur = 0;

      const grad2 = ctx.createLinearGradient(AI_X, 0, AI_X + PADDLE_W, 0);
      grad2.addColorStop(0, "#FF5252");
      grad2.addColorStop(1, "#D32F2F");
      ctx.fillStyle = grad2;
      ctx.shadowColor = "#FF5252";
      ctx.shadowBlur = 10;
      ctx.fillRect(AI_X, this.ai_y - PADDLE_H / 2, PADDLE_W, PADDLE_H);
      ctx.shadowBlur = 0;

      ctx.shadowColor = "#FFD600";
      ctx.shadowBlur = 20;
      ctx.fillStyle = "#FFD600";
      ctx.beginPath();
      ctx.arc(this.ball_x, this.ball_y, BALL_R, 0, Math.PI * 2);
      ctx.fill();
      ctx.shadowBlur = 0;

      ctx.fillStyle = "rgba(255, 214, 0, 0.08)";
      ctx.beginPath();
      ctx.arc(this.ball_x, this.ball_y, 18, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = "#ffffff";
      ctx.font = "bold 48px 'Segoe UI', monospace";
      ctx.textAlign = "center";
      ctx.textBaseline = "top";
      ctx.fillText(this.player_score, W / 4, 20);
      ctx.fillText(this.ai_score, 3 * W / 4, 20);

      ctx.fillStyle = "#666";
      ctx.font = "14px monospace";
      ctx.textBaseline = "bottom";
      ctx.fillText(`rally: ${this.rally}`, W / 2, H - 8);

      if (this.done && this.winner) {
        ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
        ctx.fillRect(0, 0, W, H);
        ctx.fillStyle = "#FFD600";
        ctx.font = "bold 64px 'Segoe UI', sans-serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(`🏆 ${this.winner} 獲勝！`, W / 2, H / 2 - 30);
        ctx.fillStyle = "#aaa";
        ctx.font = "20px monospace";
        ctx.fillText(`${this.player_score} - ${this.ai_score}`, W / 2, H / 2 + 40);
        ctx.fillStyle = "#888";
        ctx.font = "14px monospace";
        ctx.fillText("按 R 重新開始", W / 2, H / 2 + 80);
      }
    }

    title() { return "Pong"; }
  }

  // ── Assault (space shooter) ────────────────────
  const A_W = 800, A_H = 500;

  class Assault extends Game {
    init() {
      this.score = 0;
      this.lives = 3;
      this.wave = 0;
      this.player_x = A_W / 2;
      this.bullets = [];
      this.enemy_bullets = [];
      this.enemies = [];
      this.fire_cooldown = 0;
      this.game_over = false;
      this.wave_display = 0;
      this.start_wave(1);
    }

    start_wave(n) {
      this.wave = n;
      this.enemies = [];
      this.bullets = [];
      this.enemy_bullets = [];
      this.wave_display = 60;

      const num = 4 + n * 2;
      const cols = Math.min(num, 8);
      const rows = Math.ceil(num / cols);
      const gap = 60;
      const sx = (A_W - (cols - 1) * gap) / 2;
      let count = 0;
      for (let r = 0; r < rows && count < num; r++) {
        for (let c = 0; c < cols && count < num; c++) {
          const hp = n >= 3 && r === 0 ? 2 : 1;
          this.enemies.push({
            x: sx + c * gap, y: 60 + r * 50,
            w: hp > 1 ? 36 : 28, h: hp > 1 ? 24 : 20,
            hp, max_hp: hp,
            shoot_timer: Math.random() * 120,
            swoop_timer: Math.random() * 300,
          });
          count++;
        }
      }
      this.formation_dx = 1;
    }

    step(input) {
      if (this.game_over) return { done: false };

      if (this.wave_display > 0) { this.wave_display--; return { done: false }; }

      if (input.left) this.player_x = Math.max(this.player_x - 6, 20);
      if (input.right) this.player_x = Math.min(this.player_x + 6, A_W - 20);

      if (this.fire_cooldown > 0) this.fire_cooldown--;
      if (input.action && this.fire_cooldown === 0 && this.bullets.length < 4) {
        this.bullets.push({ x: this.player_x, y: 460 });
        this.fire_cooldown = 12;
      }

      for (const b of this.bullets) b.y -= 10;
      this.bullets = this.bullets.filter(b => b.y > -20);

      for (const b of this.enemy_bullets) b.y += 5;
      this.enemy_bullets = this.enemy_bullets.filter(b => b.y < A_H + 20);

      let min_x = Infinity, max_x = -Infinity;
      for (const e of this.enemies) {
        if (e.x < min_x) min_x = e.x;
        if (e.x > max_x) max_x = e.x;
      }
      if (min_x < 20 || max_x > A_W - 20) {
        this.formation_dx *= -1;
        for (const e of this.enemies) e.y += 8;
      }

      const speed = this.formation_dx * (1 + (this.wave - 1) * 0.15);
      for (const e of this.enemies) {
        e.x += speed;
        e.swoop_timer--;
        if (e.swoop_timer <= 0) { e.y += 20; e.swoop_timer = 120 + Math.random() * 180; }
        if (e.hp > 1) {
          e.shoot_timer--;
          if (e.shoot_timer <= 0) {
            this.enemy_bullets.push({ x: e.x, y: e.y });
            e.shoot_timer = 60 + Math.random() * 120;
          }
        }
      }

      for (let i = this.bullets.length - 1; i >= 0; i--) {
        const b = this.bullets[i];
        for (let j = this.enemies.length - 1; j >= 0; j--) {
          const e = this.enemies[j];
          if (Math.abs(b.x - e.x) < e.w / 2 + 2 && Math.abs(b.y - e.y) < e.h / 2 + 6) {
            e.hp--;
            this.bullets.splice(i, 1);
            if (e.hp <= 0) {
              this.score += e.max_hp > 1 ? 200 : 100;
              this.enemies.splice(j, 1);
            }
            break;
          }
        }
      }

      for (let i = this.enemy_bullets.length - 1; i >= 0; i--) {
        const b = this.enemy_bullets[i];
        if (Math.abs(b.x - this.player_x) < 20 && Math.abs(b.y - 470) < 12) {
          this.lives--;
          this.enemy_bullets.splice(i, 1);
          if (this.lives <= 0) return this._game_over();
        }
      }

      for (let i = this.enemies.length - 1; i >= 0; i--) {
        const e = this.enemies[i];
        if (Math.abs(e.x - this.player_x) < 20 + e.w / 2 && Math.abs(e.y - 470) < 12 + e.h / 2) {
          this.lives--;
          this.enemies.splice(i, 1);
          if (this.lives <= 0) return this._game_over();
        }
      }

      this.enemies = this.enemies.filter(e => e.y < A_H - 40);
      if (this.enemies.length === 0) this.start_wave(this.wave + 1);
      return { done: false };
    }

    _game_over() {
      this.game_over = true;
      return { done: true, score: [this.score, 0], winner: "玩家", rally: this.wave };
    }

    render(ctx) {
      ctx.fillStyle = "#0a0a1a";
      ctx.fillRect(0, 0, A_W, A_H);

      ctx.fillStyle = "#1a1a3a";
      for (let i = 0; i < 60; i++) {
        ctx.fillRect((i * 137 + 50) % A_W, (i * 97 + 20) % (A_H * 0.6), 2, 2);
      }

      ctx.fillStyle = "#0f0f23";
      ctx.fillRect(0, A_H - 30, A_W, 30);
      ctx.strokeStyle = "#1a1a3a";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, A_H - 30);
      ctx.lineTo(A_W, A_H - 30);
      ctx.stroke();

      ctx.shadowColor = "#00E5FF";
      ctx.shadowBlur = 15;
      ctx.fillStyle = "#00E5FF";
      ctx.beginPath();
      ctx.moveTo(this.player_x, 455);
      ctx.lineTo(this.player_x - 20, 483);
      ctx.lineTo(this.player_x + 20, 483);
      ctx.closePath();
      ctx.fill();
      ctx.shadowBlur = 0;

      ctx.shadowColor = "#FFD600";
      ctx.shadowBlur = 10;
      ctx.fillStyle = "#FFD600";
      for (const b of this.bullets) {
        ctx.fillRect(b.x - 2, b.y - 6, 4, 12);
      }
      ctx.shadowBlur = 0;

      for (const e of this.enemies) {
        if (e.max_hp > 1) {
          ctx.shadowColor = "#9C27B0";
          ctx.shadowBlur = 12;
          ctx.fillStyle = e.hp > 1 ? "#CE93D8" : "#9C27B0";
          ctx.fillRect(e.x - e.w / 2, e.y - e.h / 2, e.w, e.h);
          ctx.fillRect(e.x - e.w, e.y - 2, 8, 4);
          ctx.fillRect(e.x + e.w - 8, e.y - 2, 8, 4);
        } else {
          ctx.shadowColor = "#FF5252";
          ctx.shadowBlur = 10;
          ctx.fillStyle = "#FF5252";
          ctx.beginPath();
          ctx.moveTo(e.x, e.y - e.h / 2);
          ctx.lineTo(e.x + e.w / 2, e.y);
          ctx.lineTo(e.x, e.y + e.h / 2);
          ctx.lineTo(e.x - e.w / 2, e.y);
          ctx.closePath();
          ctx.fill();
        }
        ctx.shadowBlur = 0;
      }

      ctx.shadowColor = "#FF5252";
      ctx.shadowBlur = 8;
      ctx.fillStyle = "#FF5252";
      for (const b of this.enemy_bullets) {
        ctx.fillRect(b.x - 2, b.y - 4, 4, 8);
      }
      ctx.shadowBlur = 0;

      ctx.fillStyle = "#e0e0e0";
      ctx.font = "16px 'Consolas', monospace";
      ctx.textAlign = "left";
      ctx.textBaseline = "top";
      ctx.fillText("SCORE: " + this.score, 10, 10);
      ctx.textAlign = "right";
      ctx.fillText("WAVE: " + this.wave, A_W - 10, 10);
      ctx.textAlign = "right";
      ctx.fillStyle = "#FF5252";
      let h = "";
      for (let i = 0; i < this.lives; i++) h += "\u2665 ";
      ctx.fillText(h.trim(), A_W - 10, 32);

      if (this.wave_display > 0) {
        ctx.fillStyle = "rgba(0,0,0,0.5)";
        ctx.fillRect(0, 0, A_W, A_H);
        ctx.fillStyle = "#FFD600";
        ctx.font = "bold 48px 'Segoe UI', sans-serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("WAVE " + this.wave, A_W / 2, A_H / 2);
      }

      if (this.game_over) {
        ctx.fillStyle = "rgba(0,0,0,0.6)";
        ctx.fillRect(0, 0, A_W, A_H);
        ctx.fillStyle = "#FFD600";
        ctx.font = "bold 56px 'Segoe UI', sans-serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("GAME OVER", A_W / 2, A_H / 2 - 40);
        ctx.fillStyle = "#e0e0e0";
        ctx.font = "28px 'Consolas', monospace";
        ctx.fillText("SCORE: " + this.score, A_W / 2, A_H / 2 + 20);
        ctx.fillStyle = "#888";
        ctx.font = "16px 'Consolas', monospace";
        ctx.fillText("\u6309 R \u91CD\u65B0\u958B\u59CB", A_W / 2, A_H / 2 + 70);
      }
    }

    title() { return "Assault"; }
  }

  return { Input, Game, Engine, draw, Vec2, Pong, Assault };
})();

if (typeof module !== "undefined" && module.exports) module.exports = game4;
