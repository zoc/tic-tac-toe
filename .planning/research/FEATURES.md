# Feature Research

**Domain:** Browser-based casual game (tic-tac-toe vs computer)
**Researched:** 2026-04-12
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Playable 3x3 grid with click/tap input | It's the game. No grid = no game. | LOW | HTML/CSS grid or canvas; must handle both mouse and touch events |
| Turn-based play (human X, computer O) | Core mechanic. Human makes a move, computer responds. | LOW | WASM handles game state; JS triggers computer move after human click |
| Win/loss/draw detection | Game must end properly. Players need to know the outcome. | LOW | 8 possible win lines to check; straightforward in Rust |
| Visual win line highlighting | Drawing a line through winning 3-in-a-row is fundamental to tic-tac-toe — it's how the game has been played on paper for centuries | MEDIUM | Animate a line/highlight across the three winning cells; CSS or SVG overlay |
| Clear game outcome message | "You win!", "Computer wins!", "It's a draw!" — without this the game feels broken | LOW | Status text area or modal; update on game end |
| New game / restart button | Players expect to play again instantly. Making them refresh = amateur hour. | LOW | Reset board state in WASM, clear UI |
| Computer opponent that actually plays | A computer that doesn't respond, plays randomly, or plays perfectly is all bad. Must feel like a real opponent. | MEDIUM | Minimax with random mistake injection; tuning the mistake rate is the real work |
| Turn indicator ("Your turn" / "Computer thinking...") | Without this, players don't know if the game is waiting for them or processing | LOW | Simple status text that updates on state change |
| Responsive layout (works on phone + desktop) | Over 50% of casual game traffic is mobile. Non-responsive = half your audience lost. | MEDIUM | CSS responsive grid; touch targets need to be large enough (min 44px) |
| Occupied cell rejection (can't overwrite a move) | Clicking a filled cell should do nothing (or show feedback). Letting players overwrite = game-breaking bug. | LOW | Check cell state before accepting move; already natural from game logic |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Score tracking across games (W/L/D) | Adds replayability — players want to "beat their record" against the computer. Most basic implementations skip this. | LOW | Counter in JS; persist in localStorage for session survival |
| Smooth CSS animations (piece placement, win celebration) | Polished feel separates this from thousands of bare-bones implementations. Animations make the game feel *alive*. | MEDIUM | CSS transitions for piece appear (scale/fade), win line draw animation, board reset transition |
| Computer "thinking" delay | Instant computer response feels robotic and unsatisfying. A brief artificial delay (300-800ms) makes it feel like the computer is deliberating. | LOW | setTimeout wrapper around WASM AI call; consider random delay range for naturalness |
| Winning celebration effect | Confetti, glow, shake, color burst — something that makes winning feel rewarding. Dopamine hit keeps players coming back. | MEDIUM | CSS keyframe animations or lightweight canvas particle effect |
| Sound effects (move placement, win/lose) | Audio feedback makes interactions feel tangible. Casual games with sound feel significantly more polished. | MEDIUM | Web Audio API or simple `<audio>` elements; needs mute toggle; keep sounds short and satisfying |
| Dark mode / theme support | Modern web expectation. Shows attention to detail. | LOW | CSS custom properties for theming; respect `prefers-color-scheme` |
| Keyboard accessibility | Tab through cells, Enter/Space to place piece. Important for accessibility and completeness. | LOW | `tabindex` on cells, keyboard event handlers, focus styling |
| Persistent scores (survive page refresh) | Players close the tab and come back — their score history is preserved. Small touch, big quality signal. | LOW | localStorage read/write; serialize score object as JSON |
| WASM-powered game engine (Rust) | Technical differentiator — the game logic runs at near-native speed via WebAssembly. Most tic-tac-toe games use plain JS. While tic-tac-toe doesn't *need* WASM performance, it demonstrates Rust/WASM capabilities and provides a clean separation between game engine and UI. | HIGH | This is the core architecture decision. Rust + wasm-pack + wasm-bindgen. High complexity due to toolchain setup, not game logic. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems. These align with PROJECT.md's Out of Scope.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Online multiplayer | "Play with friends!" is the obvious next step | Requires server infrastructure, WebSocket handling, matchmaking, latency management, disconnection handling. Massively increases scope for a game that's fundamentally simple. Tic-tac-toe games online converge to all-draws between competent players. | Single-player focus with a well-tuned AI that feels fun to play against. The beatable AI *is* the multiplayer stand-in. |
| Two-player local (pass-and-play) | "Let me play with someone sitting next to me" | Splits focus between AI experience and 2P experience. The AI tuning, computer thinking delay, and celebration effects all need separate paths. Also, 2P tic-tac-toe between adults always ends in draws — it's not fun. | Keep single-player focus. The computer opponent with personality (mistakes, "thinking") is more interesting than a human who plays perfectly. |
| Difficulty selection (easy/medium/hard) | "Let me choose how hard the AI is" | Adds UI complexity (settings screen), requires tuning multiple difficulty levels, and "hard" on tic-tac-toe is just "never loses" which is frustrating. | Single beatable difficulty that plays well but makes occasional mistakes. One well-tuned experience beats three mediocre ones. |
| Player choosing X or O | "Let me pick my symbol" | Adds pregame UI flow, complicates turn order logic, and the choice is purely cosmetic — it doesn't change gameplay. | Human is always X, always goes first. Simpler UX, faster to start playing. |
| Move history / undo | React tutorial's "time travel" feature. Seems educational and useful. | Over-engineering for a casual game. Players want to play forward, not review moves. Adds state management complexity. Undo undermines the challenge of playing against AI. | No undo. Each game is quick (max 9 moves). Just start a new game if unhappy. |
| Leaderboards / global rankings | "Compete with other players worldwide" | Requires backend server, user accounts, anti-cheat measures. Tic-tac-toe doesn't have meaningful skill differentiation for rankings. | Local score tracking is sufficient. "Beat your own record" against the computer. |
| Board size options (4x4, 5x5) | "More complexity = more fun" | Changes the fundamental game. 4x4+ tic-tac-toe has very different strategy. Minimax becomes computationally expensive. The entire AI needs reworking. | Keep 3x3. It's what people expect from "tic-tac-toe." Variations are different games entirely. |
| Achievements / badges system | Gamification is trendy | Over-engineering for a simple game. Adds persistent state management, UI for achievement display, trigger logic. The game loop is already ~30 seconds per game. | Score tracking provides enough meta-game. Win streaks are a natural achievement. |

## Feature Dependencies

```
[Playable Grid]
    └──requires──> [Game State Engine (WASM)]
                       └──requires──> [Win/Loss/Draw Detection]
                                          └──enables──> [Win Line Highlighting]
                                          └──enables──> [Game Outcome Message]
                                          └──enables──> [Score Tracking]

[Computer Opponent (AI)]
    └──requires──> [Game State Engine (WASM)]
    └──enhanced-by──> [Thinking Delay]

[New Game Button]
    └──requires──> [Game State Engine (WASM)]
    └──resets──> [Playable Grid]

[Score Tracking]
    └──enhanced-by──> [Persistent Scores (localStorage)]

[Animations]
    └──enhances──> [Playable Grid] (piece placement)
    └──enhances──> [Win Line Highlighting] (win celebration)
    └──enhances──> [New Game Button] (board reset transition)

[Sound Effects]
    └──enhances──> [Playable Grid] (move sound)
    └──enhances──> [Win Line Highlighting] (win/lose sound)
    └──independent (can be added anytime)]

[Responsive Layout]
    └──affects──> [Playable Grid] (touch targets, sizing)
    └──independent (CSS layer, no logic dependency)]
```

### Dependency Notes

- **Win Line Highlighting requires Win Detection:** Must know which 3 cells won before you can highlight them. The WASM engine should return the winning line coordinates, not just a boolean.
- **Score Tracking requires Win Detection:** Can only increment W/L/D counters after game outcome is determined.
- **Computer AI requires Game State Engine:** AI evaluates the board state to choose its move. Both live in Rust/WASM.
- **Thinking Delay enhances AI:** The delay wraps the AI call on the JS side — the WASM returns instantly, JS adds the delay for UX.
- **Animations are independent layers:** CSS animations layer on top of game logic. Can be added progressively without changing core functionality.
- **Sound Effects are fully independent:** Can be added at any phase without modifying game logic or UI structure.

## MVP Definition

### Launch With (v1)

Minimum viable product — a complete, playable, polished game.

- [ ] Rust/WASM game engine (board state, move validation, win detection) — the core technical foundation
- [ ] 3x3 grid rendered in browser with click/tap input — the game surface
- [ ] Computer opponent with beatable AI (imperfect minimax) — the single-player experience
- [ ] Win/loss/draw detection with winning line highlighting — game completion feedback
- [ ] Game outcome message ("You win!", etc.) — clear communication of results
- [ ] Turn indicator — so the player always knows what's happening
- [ ] New game / restart button — instant replay
- [ ] Score tracking (wins, losses, draws) — replayability hook
- [ ] Responsive layout — works on phone and desktop
- [ ] Occupied cell rejection — basic input validation

### Add After Validation (v1.x)

Features to add once core game is solid and fun.

- [ ] CSS animations (piece placement, board transitions) — adds polish and life to the game
- [ ] Win celebration effect (confetti, glow, or similar) — makes winning feel rewarding
- [ ] Computer "thinking" delay (300-800ms) — makes the AI feel deliberate, not robotic
- [ ] Sound effects with mute toggle — tactile audio feedback
- [ ] Persistent scores via localStorage — scores survive page refresh
- [ ] Dark mode / prefers-color-scheme support — modern web standard

### Future Consideration (v2+)

Features to defer until the core experience is validated.

- [ ] Keyboard accessibility (tab navigation, enter to place) — important for a11y but not launch-blocking
- [ ] Animated SVG X and O pieces (hand-drawn style) — distinctive visual identity
- [ ] Tutorial/hint system for new players — gentle guidance on strategy

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Playable 3x3 grid | HIGH | LOW | P1 |
| WASM game engine | HIGH | HIGH | P1 |
| Computer opponent (AI) | HIGH | MEDIUM | P1 |
| Win/loss/draw detection | HIGH | LOW | P1 |
| Win line highlighting | HIGH | MEDIUM | P1 |
| Game outcome message | HIGH | LOW | P1 |
| Turn indicator | MEDIUM | LOW | P1 |
| New game button | HIGH | LOW | P1 |
| Responsive layout | HIGH | MEDIUM | P1 |
| Score tracking | MEDIUM | LOW | P1 |
| CSS animations | MEDIUM | MEDIUM | P2 |
| Thinking delay | MEDIUM | LOW | P2 |
| Win celebration | MEDIUM | MEDIUM | P2 |
| Sound effects | LOW | MEDIUM | P2 |
| Persistent scores | LOW | LOW | P2 |
| Dark mode | LOW | LOW | P2 |
| Keyboard accessibility | MEDIUM | LOW | P3 |

**Priority key:**
- P1: Must have for launch — without these, it's not a game
- P2: Should have, adds polish — the difference between "demo" and "product"
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | playtictactoe.org (Neave) | Google Search TTT | PaperGames.io | Our Approach |
|---------|---------------------------|-------------------|---------------|--------------|
| Grid rendering | Clean retro SVG | Inline game widget | Full web app | CSS grid with animations |
| Computer AI | Yes, difficulty unclear | Yes, 3 difficulty levels | Yes, online only | Beatable imperfect minimax |
| Score tracking | Yes (session) | No | Yes (with accounts) | Yes, local + persistent |
| Win highlighting | Yes, line animation | Yes, line through winning cells | Yes | Animated line overlay |
| Two-player mode | Yes (local) | Yes (local) | Yes (online) | No — deliberate single-player focus |
| Animations | Minimal | Minimal | Moderate | Rich (our differentiator) |
| Sound effects | No | No | No | Yes (our differentiator) |
| Difficulty options | No | Easy/Medium/Impossible | Via opponent skill | No — single well-tuned level |
| Mobile support | Yes | Yes | Yes | Yes, responsive-first |
| Visual polish | Retro aesthetic, clean | Basic Google UI | Gaming site aesthetic | Modern, polished, animated |
| Technology | JavaScript | JavaScript | JavaScript | Rust/WASM (unique) |

**Key insight:** Most existing tic-tac-toe web games are visually basic. They nail the gameplay but skimp on polish (no animations, no sound, minimal celebration). Our opportunity is to deliver the same solid gameplay with noticeably more visual and audio polish — making it feel like a *product* rather than a *tutorial exercise*. The Rust/WASM architecture is a unique technical differentiator even if end-users don't directly perceive it.

## Sources

- playtictactoe.org (Neave Interactive) — Leading browser tic-tac-toe implementation
- PaperGames.io/tic-tac-toe — Feature-rich online gaming platform version
- React.dev tutorial — Classic tic-tac-toe implementation with move history
- Wikipedia: Tic-tac-toe — Game theory, strategy, solved game analysis
- MDN Web Docs: Introduction to Game Development — Web platform capabilities for games
- Google Search embedded tic-tac-toe game — Mass-market baseline

---
*Feature research for: browser-based tic-tac-toe (Rust/WASM)*
*Researched: 2026-04-12*
