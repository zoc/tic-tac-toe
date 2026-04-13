// src/main.js
// Tic-Tac-Toe browser game — Phase 3 + v1.1 Polish
// Connects WASM game engine (Phase 2) to DOM (index.html + style.css)

import init, { WasmGame } from '../pkg/tic_tac_toe.js';
import { sounds, toggleMute, isMuted } from './audio.js';

// ─── Constants ────────────────────────────────────────────────────────────────
const PLAYER_X = 1;   // human
const PLAYER_O = 2;   // computer
const NO_MOVE  = 255; // sentinel: computer_move() returns this when game is over

// ─── Score persistence helpers (localStorage) ────────────────────────────────
const SCORE_KEY = 'ttt-score';

function loadScore() {
  try {
    const saved = localStorage.getItem(SCORE_KEY);
    return saved ? JSON.parse(saved) : { wins: 0, losses: 0, draws: 0 };
  } catch {
    return { wins: 0, losses: 0, draws: 0 };  // SecurityError in private browsing
  }
}

function saveScore() {
  try {
    localStorage.setItem(SCORE_KEY, JSON.stringify(score));
  } catch {
    // Storage quota exceeded or unavailable — silently ignore
  }
}

// ─── In-memory score — loaded from localStorage, persists across refreshes ───
const score = loadScore();

// ─── DOM references (queried once at startup) ─────────────────────────────────
const boardEl      = document.getElementById('board');
const statusEl     = document.getElementById('status-message');
const restartBtn   = document.getElementById('restart-btn');
const muteBtn      = document.getElementById('mute-btn');
const scoreWinsEl  = document.getElementById('score-wins');
const scoreLossEl  = document.getElementById('score-losses');
const scoreDrawEl  = document.getElementById('score-draws');
const winLineEl    = document.getElementById('win-line');

// ─── Win line: lookup table of sorted positions → CSS class ──────────────────
const WIN_LINE_CLASSES = {
  '0,1,2': 'win-line--row0',
  '3,4,5': 'win-line--row1',
  '6,7,8': 'win-line--row2',
  '0,3,6': 'win-line--col0',
  '1,4,7': 'win-line--col1',
  '2,5,8': 'win-line--col2',
  '0,4,8': 'win-line--diag-lr',
  '2,4,6': 'win-line--diag-rl',
};

function showWinLine(positions) {
  const key = [...positions].sort((a, b) => a - b).join(',');
  const cls = WIN_LINE_CLASSES[key];
  if (!cls) return;
  winLineEl.className = `win-line ${cls}`;
  winLineEl.hidden = false;
}

function clearWinLine() {
  winLineEl.hidden = true;
  winLineEl.className = 'win-line';
}

// ─── Game state ───────────────────────────────────────────────────────────────
let game;              // WasmGame instance — created after init()
let isProcessing = false; // guard against double-click during computer move

// ─── Computer thinking delay ──────────────────────────────────────────────────
const THINK_MIN = 300;
const THINK_MAX = 800;
function thinkDelay() {
  const ms = THINK_MIN + Math.random() * (THINK_MAX - THINK_MIN);
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ─── Render: draw the full board from WASM state ─────────────────────────────
function renderBoard(winningPositions = []) {
  const board = game.get_board();  // Uint8Array[9]

  // Build a Set for O(1) winning-cell lookup
  const winSet = new Set(winningPositions);

  boardEl.innerHTML = '';

  for (let i = 0; i < 9; i++) {
    const cell = document.createElement('div');
    cell.className = 'cell';
    cell.dataset.index = String(i);
    cell.setAttribute('role', 'gridcell');

    const value = board[i];

    if (value === PLAYER_X) {
      cell.textContent = 'X';
      cell.classList.add('cell--taken', 'cell--x');
      cell.setAttribute('aria-label', `X at position ${i + 1}`);
    } else if (value === PLAYER_O) {
      cell.textContent = 'O';
      cell.classList.add('cell--taken', 'cell--o');
      cell.setAttribute('aria-label', `O at position ${i + 1}`);
    } else {
      cell.setAttribute('aria-label', `Empty cell ${i + 1}`);
      // Keyboard accessibility (LW-01): empty cells can receive focus and be activated
      cell.tabIndex = 0;
      cell.setAttribute('role', 'button');
    }

    if (winSet.has(i)) {
      cell.classList.add('cell--winning');
    }

    boardEl.appendChild(cell);
  }
}

// ─── Update status message and its CSS modifier class ────────────────────────
function setStatus(text, modifier = '') {
  statusEl.textContent = text;
  statusEl.className = 'status-message' + (modifier ? ` status--${modifier}` : '');
}

// ─── Update the score display ─────────────────────────────────────────────────
function updateScoreDisplay() {
  scoreWinsEl.textContent = String(score.wins);
  scoreLossEl.textContent = String(score.losses);
  scoreDrawEl.textContent = String(score.draws);
}

// ─── Handle end-of-game: highlight, message, update score ─────────────────────
function handleGameOver() {
  const status = game.get_status();
  const winner = game.get_winner();
  const winPositions = status === 'won' ? Array.from(game.get_winning_positions()) : [];

  // Render board with winning cells highlighted (Decision D)
  renderBoard(winPositions);

  // Show animated win line through winning cells
  if (winPositions.length) showWinLine(winPositions);

  // Update status message (UI-03)
  if (status === 'won') {
    if (winner === PLAYER_X) {
      setStatus('You win! 🎉', 'win');
      sounds.win();
      score.wins++;
    } else {
      setStatus('Computer wins!', 'loss');
      sounds.loss();
      score.losses++;
    }
  } else {
    setStatus("It's a draw!", 'draw');
    sounds.draw();
    score.draws++;
  }

  saveScore();          // persist across page refreshes
  updateScoreDisplay();  // SCORE-01

  // Show restart button (UI-04)
  restartBtn.hidden = false;

  // Disable board interaction — add class to prevent hover states
  boardEl.classList.add('board--disabled');
}

// ─── Handle a human click on a cell ──────────────────────────────────────────
async function handleCellClick(event) {
  // Guard: ignore if processing (computer move in flight) or game is over
  if (isProcessing) return;
  if (game.get_status() !== 'playing') return;
  if (game.current_player() !== PLAYER_X) return;

  const cell = event.target.closest('.cell');
  if (!cell) return;

  const index = parseInt(cell.dataset.index, 10);
  if (isNaN(index)) return;

  // Attempt human move (UI-01 + ENG-02)
  const moved = game.make_move(index);
  if (!moved) return;  // occupied cell — ignore silently

  sounds.move();  // feedback for human piece placement
  renderBoard();

  // Check if human won after their move
  if (game.get_status() !== 'playing') {
    handleGameOver();
    return;
  }

  // ── Computer's turn ──────────────────────────────────────────────────────
  // Set processing flag BEFORE delay to block any stray clicks (Decision C)
  isProcessing = true;
  boardEl.classList.add('board--disabled');
  setStatus("Computer's turn", 'computer-turn');  // UI-02

  // Artificial thinking delay — makes the computer feel responsive, not instant
  await thinkDelay();

  const compPos = game.computer_move();

  if (compPos === NO_MOVE) {
    // Should not happen (we already checked status === 'playing'), but be safe
    isProcessing = false;
    boardEl.classList.remove('board--disabled');
    setStatus('Your turn');  // restore correct status — prevents stale "Computer's turn" message
    return;
  }

  sounds.computerMove();  // feedback for computer piece placement
  renderBoard();
  isProcessing = false;

  // Check if computer won or drew after its move
  if (game.get_status() !== 'playing') {
    handleGameOver();
    return;
  }

  // Game continues — restore human turn state
  boardEl.classList.remove('board--disabled');
  setStatus('Your turn');  // UI-02
}

// ─── Reset game (UI-04) ───────────────────────────────────────────────────────
function resetGame() {
  game.reset();
  isProcessing = false;
  boardEl.classList.remove('board--disabled');
  restartBtn.hidden = true;
  clearWinLine();
  renderBoard();
  setStatus('Your turn');
}

// ─── Entry point: init WASM, then wire events ─────────────────────────────────
async function main() {
  // WASM must be initialized before any WasmGame calls (see PITFALLS.md Pitfall 1)
  await init();

  game = new WasmGame();

  // Render empty board
  renderBoard();
  setStatus('Your turn');
  updateScoreDisplay();

  // Single delegated click listener on board — handles all 9 cells (UI-01)
  boardEl.addEventListener('click', handleCellClick);

  // Keyboard navigation (LW-01): Enter or Space activates the focused cell
  boardEl.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleCellClick(e);
    }
  });

  // Restart button (UI-04)
  restartBtn.addEventListener('click', resetGame);

  // Mute toggle button — persisted to localStorage via audio.js
  muteBtn.textContent = isMuted() ? '🔇' : '🔊';
  muteBtn.setAttribute('aria-pressed', String(isMuted()));
  muteBtn.addEventListener('click', () => {
    const nowMuted = toggleMute();
    muteBtn.textContent = nowMuted ? '🔇' : '🔊';
    muteBtn.setAttribute('aria-pressed', String(nowMuted));
  });
}

main().catch(err => {
  // Graceful error display if WASM fails to load — build DOM imperatively to avoid XSS
  const wrapper = document.createElement('div');
  wrapper.className = 'error-overlay';
  const heading = document.createElement('h2');
  heading.textContent = 'Failed to load game';
  const pre = document.createElement('pre');
  pre.textContent = err.message;          // safe: textContent, never innerHTML
  wrapper.append(heading, pre);
  document.body.replaceChildren(wrapper);
});
