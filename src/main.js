// src/main.js
// Tic-Tac-Toe browser game — Phase 3
// Connects WASM game engine (Phase 2) to DOM (index.html + style.css)

import init, { WasmGame } from '../pkg/tic_tac_toe.js';

// ─── Constants ────────────────────────────────────────────────────────────────
const PLAYER_X = 1;   // human
const PLAYER_O = 2;   // computer
const NO_MOVE  = 255; // sentinel: computer_move() returns this when game is over

// ─── In-memory score (SCORE-01) — resets on page refresh (per CONTEXT.md) ────
const score = { wins: 0, losses: 0, draws: 0 };

// ─── DOM references (queried once at startup) ─────────────────────────────────
const boardEl      = document.getElementById('board');
const statusEl     = document.getElementById('status-message');
const restartBtn   = document.getElementById('restart-btn');
const scoreWinsEl  = document.getElementById('score-wins');
const scoreLossEl  = document.getElementById('score-losses');
const scoreDrawEl  = document.getElementById('score-draws');

// ─── Game state ───────────────────────────────────────────────────────────────
let game;              // WasmGame instance — created after init()
let isProcessing = false; // guard against double-click during computer move

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

  // Update status message (UI-03)
  if (status === 'won') {
    if (winner === PLAYER_X) {
      setStatus('You win! 🎉', 'win');
      score.wins++;
    } else {
      setStatus('Computer wins!', 'loss');
      score.losses++;
    }
  } else {
    setStatus("It's a draw!", 'draw');
    score.draws++;
  }

  updateScoreDisplay();  // SCORE-01

  // Show restart button (UI-04)
  restartBtn.hidden = false;

  // Disable board interaction — add class to prevent hover states
  boardEl.classList.add('board--disabled');
}

// ─── Handle a human click on a cell ──────────────────────────────────────────
function handleCellClick(event) {
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

  renderBoard();

  // Check if human won after their move
  if (game.get_status() !== 'playing') {
    handleGameOver();
    return;
  }

  // ── Computer's turn ──────────────────────────────────────────────────────
  // Set processing flag BEFORE computer move to block any stray clicks (Decision C: instant)
  isProcessing = true;
  boardEl.classList.add('board--disabled');
  setStatus("Computer's turn", 'computer-turn');  // UI-02

  // Computer move is synchronous — no artificial delay (per CONTEXT.md Decision C)
  const compPos = game.computer_move();

  if (compPos === NO_MOVE) {
    // Should not happen (we already checked status === 'playing'), but be safe
    isProcessing = false;
    boardEl.classList.remove('board--disabled');
    return;
  }

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

  // Restart button (UI-04)
  restartBtn.addEventListener('click', resetGame);
}

main().catch(err => {
  // Graceful error display if WASM fails to load
  document.body.innerHTML = `
    <div style="color:#e94560;padding:40px;font-family:monospace;background:#1a1a2e;min-height:100vh">
      <h2>Failed to load game</h2>
      <pre>${err.message}</pre>
    </div>`;
});
