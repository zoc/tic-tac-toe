// src/audio.js
// Synthesised sound effects via Web Audio API — no audio files needed.
// All sounds are generated from oscillator tones.
//
// Autoplay policy: AudioContext is created lazily on the first sound call,
// which is always triggered by a user gesture (cell click or mute button).

const MUTE_KEY = 'ttt-muted';

let audioCtx = null;

// Read mute preference from localStorage on module load (IIFE — runs once)
let muted = (() => {
  try { return localStorage.getItem(MUTE_KEY) === 'true'; }
  catch { return false; }  // SecurityError in private browsing
})();

// ─── Lazy AudioContext singleton ──────────────────────────────────────────────
function getCtx() {
  if (!audioCtx) audioCtx = new AudioContext();
  // Browser may auto-suspend context (Safari especially strict about this);
  // resume() returns a Promise but we don't need to await it here — the context
  // will be running by the time the oscillator nodes are scheduled.
  if (audioCtx.state !== 'running') audioCtx.resume();
  return audioCtx;
}

// ─── Primitive tone generator ─────────────────────────────────────────────────
// @param {number} frequency   - Hz
// @param {string} type        - OscillatorType ('sine'|'square'|'sawtooth'|'triangle')
// @param {number} duration    - seconds
// @param {number} gain        - peak amplitude (0.0–1.0)
function playTone({ frequency, type = 'sine', duration = 0.1, gain = 0.25 }) {
  if (muted) return;
  const ctx = getCtx();
  const osc = ctx.createOscillator();
  const gainNode = ctx.createGain();
  osc.connect(gainNode);
  gainNode.connect(ctx.destination);
  osc.type = type;
  osc.frequency.value = frequency;
  // Fade to near-zero to avoid click/pop at note end
  gainNode.gain.setValueAtTime(gain, ctx.currentTime);
  gainNode.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + duration);
}

// ─── Named sound effects ──────────────────────────────────────────────────────
export const sounds = {
  // Human places a piece — clean mid-frequency click
  move: () => playTone({ frequency: 440, type: 'sine', duration: 0.08, gain: 0.2 }),

  // Computer places a piece — slightly lower pitch to distinguish
  computerMove: () => playTone({ frequency: 330, type: 'sine', duration: 0.08, gain: 0.2 }),

  // Human wins — ascending three-note fanfare
  win: () => {
    playTone({ frequency: 523, duration: 0.12, gain: 0.3 });
    setTimeout(() => playTone({ frequency: 659, duration: 0.15, gain: 0.3 }), 80);
    setTimeout(() => playTone({ frequency: 784, duration: 0.25, gain: 0.3 }), 180);
  },

  // Computer wins — descending buzz
  loss: () => {
    playTone({ frequency: 280, type: 'sawtooth', duration: 0.15, gain: 0.2 });
    setTimeout(() => playTone({ frequency: 200, type: 'sawtooth', duration: 0.3, gain: 0.15 }), 100);
  },

  // Draw — neutral short tone
  draw: () => playTone({ frequency: 360, type: 'sine', duration: 0.2, gain: 0.2 }),
};

// ─── Mute toggle ──────────────────────────────────────────────────────────────
// Returns the new muted state (true = muted, false = unmuted)
export function toggleMute() {
  muted = !muted;
  try { localStorage.setItem(MUTE_KEY, String(muted)); } catch { /* ignore */ }
  return muted;
}

export function isMuted() { return muted; }
