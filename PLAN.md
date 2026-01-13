# PROJECT NEURON
## N64 Neural Network Visualization System
### Codename: DEEP PADDLE

---

```
╔══════════════════════════════════════════════════════════════════════════════╗
║  CLASSIFICATION: EDUCATIONAL / OPEN SOURCE                                   ║
║  PLATFORM: Nintendo 64 (4MB RAM / 64MB ROM / 93.75MHz MIPS R4300i)          ║
║  OBJECTIVE: Real-time neural network training visualization via Pong RL     ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 1. EXECUTIVE SUMMARY

**Mission**: Build an interactive, visually rich demonstration of neural network learning that runs on Nintendo 64 hardware. Users observe a neural network learn to play Pong from zero competence to superhuman performance, with full visualization of weights, activations, gradients, and loss curves in real-time.

**Why N64**: 
- Proves AI fundamentals run on 1996 hardware
- Physical artifact > web demo for memorability  
- Constraint breeds creativity in visualization
- "If an N64 can do this, demystify AI" messaging

**Inspiration**: [sgd.fyi](https://sgd.fyi/) for loss/weight visualization, but embodied in gameplay rather than abstract function fitting.

---

## 2. VISUAL IDENTITY: "SKUNKWORKS TERMINAL"

### 2.1 Design Language

```
┌─────────────────────────────────────────────────────────────────┐
│ AESTHETIC: Cold War signals intelligence meets 90s hacker film │
│                                                                 │
│ References:                                                     │
│ - NORAD war room displays (WarGames, 1983)                     │
│ - Submarine sonar screens                                       │
│ - Bloomberg terminal density                                    │
│ - Fallout Pip-Boy interface                                     │
│ - Alien (1979) Nostromo computer UI                            │
│ - Real NRO/NSA declassified system screenshots                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Color Palette (N64 RGB values)

```
PRIMARY PALETTE:
┌────────────────┬───────────┬─────────────────────────────────┐
│ Name           │ RGB       │ Usage                           │
├────────────────┼───────────┼─────────────────────────────────┤
│ VOID           │ 0,0,0     │ Background, negative space      │
│ TERMINAL_GREEN │ 0,255,65  │ Primary text, positive weights  │
│ AMBER_WARN     │ 255,176,0 │ Warnings, secondary data        │
│ PHOSPHOR_DIM   │ 0,90,25   │ Inactive elements, grid lines   │
│ HOT_WHITE      │ 255,255,255│ Peak activations, focus        │
│ NEGATIVE_RED   │ 255,45,45 │ Negative weights, loss spike    │
│ COOL_BLUE      │ 45,145,255│ Input layer, state data         │
└────────────────┴───────────┴─────────────────────────────────┘

ACTIVATION HEAT RAMP (neuron firing intensity):
  0.0 ░░░░░ PHOSPHOR_DIM
  0.2 ▒▒▒▒▒ dark green
  0.5 ▓▓▓▓▓ TERMINAL_GREEN  
  0.8 █████ bright green
  1.0 ■■■■■ HOT_WHITE (bloom effect)
```

### 2.3 Typography & Elements

```
FONTS:
- Primary: Fixed-width, 8x8 pixel grid (built-in N64 debug font or custom)
- Numbers: Dot-matrix style for readouts
- Headers: ALL CAPS, letter-spaced

UI ELEMENTS:
┌──────────┐  ╔══════════╗  ┌──────────┐
│ Standard │  ║ CRITICAL ║  │▓▓▓▓░░░░░░│  
│ Panel    │  ║ Panel    ║  │ Progress │
└──────────┘  ╚══════════╝  └──────────┘

DECORATIVE:
- Scan lines (every 2nd row at 10% opacity)
- Corner brackets on focus elements: ┌─    ─┐
                                     └─    ─┘
- Blinking cursor on active inputs: █ → _ → █
- Data stream waterfall effects in margins
```

### 2.4 Screen Layout (320x240 NTSC)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ HEADER BAR ──────────────────────────────────────────────────────────┐  │
│ │ NEURON v1.0 ░░ EPOCH 00847 ░░ ε:0.15 ░░ η:0.001 ░░ [TRAINING]       │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│ ┌─ GAME VIEWPORT ─────────────────┐  ┌─ NETWORK TOPOLOGY ──────────────┐  │
│ │                                 │  │                                  │  │
│ │     ○                           │  │   I0 ──┬── H0 ──┬── O0 [UP]     │  │
│ │    /                      █     │  │   I1 ──┼── H1 ──┼── O1 [STAY]   │  │
│ │   /                       █     │  │   I2 ──┼── H2 ──┼── O2 [DOWN]   │  │
│ │  ●                        █     │  │   I3 ──┼── H3 ──┘               │  │
│ │                                 │  │   I4 ──┼── H4                    │  │
│ │  █                              │  │   I5 ──┴── ...                   │  │
│ │  █                              │  │                                  │  │
│ │  █                              │  │  [LINE THICKNESS = |WEIGHT|]     │  │
│ │                                 │  │  [NODE BRIGHTNESS = ACTIVATION]  │  │
│ └─────────────────────────────────┘  └──────────────────────────────────┘  │
│                                                                             │
│ ┌─ LOSS CURVE ──────────────────────────────────────────────────────────┐  │
│ │ LOSS                                                                   │  │
│ │ 2.0├╮                                                                  │  │
│ │    │ ╲                                                                 │  │
│ │ 1.0├  ╲___                                                             │  │
│ │    │      ╲____╱╲___________                                           │  │
│ │ 0.0├───────────────────────────────────────────────────────▶ EPOCH    │  │
│ └────┴───────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│ ┌─ METRICS ────────────┐  ┌─ WEIGHT HISTOGRAM ─┐  ┌─ CONTROLS ──────────┐  │
│ │ WIN RATE:  87.3%     │  │     ▁▂▄█▇▄▂▁       │  │ A: Speed+  B: Speed-│  │
│ │ AVG RALLY: 12.4      │  │    -1    0    +1   │  │ START: Menu         │  │
│ │ REWARD/EP: +0.73     │  │    [DISTRIBUTION]  │  │ Z: Play vs AI       │  │
│ └──────────────────────┘  └────────────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. SYSTEM ARCHITECTURE

### 3.1 High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           N64 CARTRIDGE (ROM)                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Boot Code   │  │ Game Logic  │  │ NN Engine   │  │ Asset Data          │ │
│  │ OS Init     │  │ Pong Sim    │  │ Forward     │  │ Fonts, Sprites      │ │
│  │             │  │ Physics     │  │ Backprop    │  │ UI Elements         │ │
│  │ ~8KB        │  │ ~16KB       │  │ ~32KB       │  │ ~64KB               │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ PRE-BAKED DEMONSTRATION WEIGHTS (Optional)                              ││
│  │ - Untrained random init                                                 ││
│  │ - "10 minute" checkpoint                                                ││
│  │ - "1 hour" checkpoint                                                   ││
│  │ - "Superhuman" checkpoint                                               ││
│  │ ~32KB total                                                             ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              N64 RAM (4MB)                                  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ NEURAL NETWORK STATE                                        ~512KB   │  │
│  │ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────────────┐  │  │
│  │ │ Weights    │ │ Gradients  │ │ Momentum   │ │ Activations Cache  │  │  │
│  │ │ (Variable) │ │ (Mirror)   │ │ (Adam)     │ │ (Per Layer)        │  │  │
│  │ └────────────┘ └────────────┘ └────────────┘ └────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ TRAINING STATE                                              ~256KB   │  │
│  │ ┌────────────────────┐ ┌────────────────────┐ ┌──────────────────┐  │  │
│  │ │ Experience Replay  │ │ Loss History       │ │ Episode Stats    │  │  │
│  │ │ Buffer (Ring)      │ │ (Ring Buffer)      │ │                  │  │  │
│  │ │ 2000 transitions   │ │ 1000 points        │ │                  │  │  │
│  │ └────────────────────┘ └────────────────────┘ └──────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ VISUALIZATION STATE                                         ~128KB   │  │
│  │ ┌────────────────┐ ┌────────────────┐ ┌──────────────────────────┐  │  │
│  │ │ Weight History │ │ Activation     │ │ Gradient Flow            │  │  │
│  │ │ (For Animation)│ │ Snapshots      │ │ Vectors                  │  │  │
│  │ └────────────────┘ └────────────────┘ └──────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ FRAME BUFFERS (Double-buffered) + GAME STATE               ~1.5MB   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ FREE / HEADROOM                                            ~1.5MB   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Network Architecture Tiers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         NETWORK COMPLEXITY TIERS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─ TIER 0: MINIMAL ───────────────────────────────────────────────────┐   │
│  │ "Proof of concept - learns but limited ceiling"                      │   │
│  │                                                                      │   │
│  │ Architecture: 6 → 16 → 3                                            │   │
│  │ Parameters:   6×16 + 16 + 16×3 + 3 = 163 params                     │   │
│  │ Memory:       ~2KB (weights + gradients + momentum)                  │   │
│  │ Learn Time:   ~15 minutes to competent                               │   │
│  │ Ceiling:      Good but beatable by skilled human                     │   │
│  │                                                                      │   │
│  │ Visualization: Very clear, every weight visible                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─ TIER 1: LIGHT ─────────────────────────────────────────────────────┐   │
│  │ "Default experience - good learning curve"                           │   │
│  │                                                                      │   │
│  │ Architecture: 6 → 32 → 32 → 3                                       │   │
│  │ Parameters:   6×32 + 32 + 32×32 + 32 + 32×3 + 3 = 1,379 params      │   │
│  │ Memory:       ~16KB                                                  │   │
│  │ Learn Time:   ~30 minutes to competent, ~1 hour to strong            │   │
│  │ Ceiling:      Beats most humans consistently                         │   │
│  │                                                                      │   │
│  │ Visualization: Clean, all weights visible but denser                 │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─ TIER 2: MEDIUM ────────────────────────────────────────────────────┐   │
│  │ "Extended play - slower start, higher ceiling"                       │   │
│  │                                                                      │   │
│  │ Architecture: 6 → 64 → 64 → 32 → 3                                  │   │
│  │ Parameters:   ~6,819 params                                          │   │
│  │ Memory:       ~80KB                                                  │   │
│  │ Learn Time:   ~1 hour to competent, ~3 hours to strong               │   │
│  │ Ceiling:      Near-perfect play                                      │   │
│  │                                                                      │   │
│  │ Visualization: Aggregate view, sample weights, heat maps             │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─ TIER 3: HEAVY ─────────────────────────────────────────────────────┐   │
│  │ "Overnight training - maximum capability"                            │   │
│  │                                                                      │   │
│  │ Architecture: 6 → 128 → 128 → 64 → 32 → 3                           │   │
│  │ Parameters:   ~27,171 params                                         │   │
│  │ Memory:       ~320KB                                                 │   │
│  │ Learn Time:   ~3 hours to competent, 8+ hours to optimal             │   │
│  │ Ceiling:      Theoretically perfect (superhuman)                     │   │
│  │                                                                      │   │
│  │ Visualization: Layer summaries, statistical views                    │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─ TIER 4: SUPERHEAVY ────────────────────────────────────────────────┐   │
│  │ "Research mode - push the hardware"                                  │   │
│  │                                                                      │   │
│  │ Architecture: 6 → 256 → 256 → 128 → 64 → 3                          │   │
│  │ Parameters:   ~107,395 params                                        │   │
│  │ Memory:       ~1.2MB                                                 │   │
│  │ Learn Time:   Very slow but maximum theoretical capability           │   │
│  │ Ceiling:      Overkill for Pong but demonstrates scaling             │   │
│  │                                                                      │   │
│  │ Visualization: Highly aggregated, focus on metrics                   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. STATE MACHINE

### 4.1 Application States

```
                              ┌─────────────┐
                              │   BOOT      │
                              │  (2 sec)    │
                              └──────┬──────┘
                                     │
                                     ▼
                              ┌─────────────┐
                              │   TITLE     │
                              │   SCREEN    │◄─────────────────────┐
                              └──────┬──────┘                      │
                                     │                             │
                                     │ START                       │
                                     ▼                             │
                              ┌─────────────┐                      │
                      ┌───────│   MAIN      │───────┐              │
                      │       │   MENU      │       │              │
                      │       └──────┬──────┘       │              │
                      │              │              │              │
              NEW GAME│      SETTINGS│       LOAD  │              │
                      │              │              │              │
                      ▼              ▼              ▼              │
               ┌──────────┐  ┌──────────┐  ┌──────────┐           │
               │ TRAINING │  │ SETTINGS │  │  LOAD    │           │
               │  SETUP   │  │  PANEL   │  │  MENU    │           │
               └────┬─────┘  └──────────┘  └────┬─────┘           │
                    │                           │                  │
                    └───────────┬───────────────┘                  │
                                │                                  │
                                ▼                                  │
           ┌────────────────────────────────────────┐              │
           │                                        │              │
           │           MAIN SIMULATION              │              │
           │                                        │              │
           │  ┌──────────────────────────────────┐  │              │
           │  │         SUB-STATES:              │  │              │
           │  │                                  │  │              │
           │  │  ┌──────────┐    ┌──────────┐   │  │              │
           │  │  │ TRAINING │◄──►│ PAUSED   │   │  │              │
           │  │  │ (Auto)   │    │          │   │  │              │
           │  │  └────┬─────┘    └──────────┘   │  │              │
           │  │       │                         │  │              │
           │  │       ▼                         │  │              │
           │  │  ┌──────────┐    ┌──────────┐   │  │              │
           │  │  │ PLAY VS  │◄──►│ WATCH    │   │  │              │
           │  │  │ AI (Z)   │    │ ONLY     │   │  │              │
           │  │  └──────────┘    └──────────┘   │  │              │
           │  │                                  │  │              │
           │  └──────────────────────────────────┘  │              │
           │                                        │              │
           │  [START] → PAUSE MENU ─────────────────┼──► EXIT ────┘
           │                       │                │
           │                       └─► SAVE        │
           │                       └─► SETTINGS    │
           │                       └─► RESET       │
           └────────────────────────────────────────┘
```

### 4.2 State Definitions

```c
typedef enum {
    STATE_BOOT,           // Logo, init hardware
    STATE_TITLE,          // Title screen, press start
    STATE_MENU_MAIN,      // New/Load/Settings/About
    STATE_MENU_SETTINGS,  // Network tier, speed, display options
    STATE_MENU_LOAD,      // Load from Controller Pak or built-in checkpoints
    STATE_TRAINING_SETUP, // Confirm settings before starting
    STATE_SIM_TRAINING,   // Auto-play, learning enabled
    STATE_SIM_PAUSED,     // Frozen, menu overlay
    STATE_SIM_PLAY,       // Human vs AI, learning disabled
    STATE_SIM_WATCH,      // AI vs AI, learning disabled (demo mode)
    STATE_SAVING,         // Writing to Controller Pak
    STATE_LOADING,        // Reading from Controller Pak
} AppState;
```

---

## 5. TRAINING SYSTEM

### 5.1 Reinforcement Learning Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEEP Q-NETWORK (DQN-LITE)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  WHY DQN:                                                                   │
│  - Stable learning (experience replay breaks correlation)                   │
│  - Clear educational narrative (Q = "quality" of action)                   │
│  - Well-documented, proven on simple games                                  │
│  - Can explain: "The network learns to predict future rewards"             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ALGORITHM PSEUDOCODE:                                                      │
│                                                                             │
│  for each frame:                                                            │
│      1. OBSERVE state s = [ball_x, ball_y, ball_vx, ball_vy,               │
│                            paddle_y, opponent_y]                            │
│                                                                             │
│      2. SELECT ACTION via ε-greedy:                                        │
│         if random() < ε:                                                    │
│             action = random_choice([UP, STAY, DOWN])                       │
│         else:                                                               │
│             action = argmax(Q_network(s))                                  │
│                                                                             │
│      3. EXECUTE action, observe reward r, new state s'                     │
│         reward = +1.0 if scored                                             │
│                  -1.0 if opponent scored                                    │
│                  +0.01 if ball moving toward opponent (shaping)            │
│                  -0.001 per frame (encourage efficiency)                   │
│                                                                             │
│      4. STORE transition (s, a, r, s', done) in replay buffer              │
│                                                                             │
│      5. SAMPLE mini-batch from replay buffer                               │
│                                                                             │
│      6. COMPUTE target:                                                    │
│         y = r + γ * max(Q_network(s')) * (1 - done)                        │
│                                                                             │
│      7. COMPUTE loss:                                                      │
│         loss = MSE(Q_network(s)[a], y)                                     │
│                                                                             │
│      8. BACKPROPAGATE and update weights                                   │
│                                                                             │
│      9. DECAY ε:                                                           │
│         ε = max(ε_min, ε * decay_rate)                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Hyperparameters (User-Adjustable)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          HYPERPARAMETER PANEL                               │
├────────────────────┬───────────────────┬────────────────────────────────────┤
│ Parameter          │ Default           │ Range / Notes                      │
├────────────────────┼───────────────────┼────────────────────────────────────┤
│ Learning Rate (η)  │ 0.001             │ 0.0001 - 0.01 (log scale)         │
│ Discount (γ)       │ 0.99              │ 0.9 - 0.999                        │
│ Epsilon Start (ε)  │ 1.0               │ Fixed                              │
│ Epsilon Min        │ 0.05              │ 0.01 - 0.1                         │
│ Epsilon Decay      │ 0.9995            │ 0.999 - 0.99999                    │
│ Batch Size         │ 32                │ 16 / 32 / 64                       │
│ Replay Buffer Size │ 2000              │ 500 - 5000                         │
│ Target Update Freq │ 100               │ 50 - 500 (steps)                   │
│ Network Tier       │ LIGHT (Tier 1)    │ 0 - 4                              │
├────────────────────┴───────────────────┴────────────────────────────────────┤
│ PRESETS:                                                                    │
│   [FAST LEARNER]  - High η, fast ε decay, small network                    │
│   [BALANCED]      - Default settings                                        │
│   [CAREFUL]       - Low η, slow ε decay, stable but slow                   │
│   [EXPERIMENTAL]  - Unlock all sliders for manual tuning                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Input State Normalization

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         STATE VECTOR DEFINITION                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Raw Game State:                                                            │
│  ┌────────────────────────────────────────┐                                │
│  │  ball_x:      0 to 320 (pixels)        │                                │
│  │  ball_y:      0 to 240 (pixels)        │                                │
│  │  ball_vx:     -8 to +8 (pixels/frame)  │                                │
│  │  ball_vy:     -8 to +8 (pixels/frame)  │                                │
│  │  ai_paddle_y: 0 to 240 (pixels)        │                                │
│  │  opp_paddle_y: 0 to 240 (pixels)       │                                │
│  └────────────────────────────────────────┘                                │
│                              │                                              │
│                              ▼                                              │
│                      NORMALIZATION                                          │
│                              │                                              │
│                              ▼                                              │
│  Normalized State (all values -1.0 to +1.0):                               │
│  ┌────────────────────────────────────────┐                                │
│  │  s[0] = (ball_x - 160) / 160           │  // Centered                   │
│  │  s[1] = (ball_y - 120) / 120           │  // Centered                   │
│  │  s[2] = ball_vx / 8                    │  // Velocity normalized        │
│  │  s[3] = ball_vy / 8                    │  // Velocity normalized        │
│  │  s[4] = (ai_paddle_y - 120) / 120      │  // Centered                   │
│  │  s[5] = (opp_paddle_y - 120) / 120     │  // Centered                   │
│  └────────────────────────────────────────┘                                │
│                                                                             │
│  OPTIONAL EXTENDED STATE (for deeper networks):                            │
│  ┌────────────────────────────────────────┐                                │
│  │  s[6] = ball_dist_to_paddle / 160      │  // Distance feature           │
│  │  s[7] = ball_y - ai_paddle_y / 120     │  // Relative position          │
│  │  s[8] = sign(ball_vx)                  │  // Ball direction             │
│  └────────────────────────────────────────┘                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. VISUALIZATION SYSTEM

### 6.1 Network Topology View

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    NETWORK TOPOLOGY VISUALIZATION                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  RENDERING APPROACH:                                                        │
│                                                                             │
│    NODES (Neurons):                                                         │
│    - Circle radius: 3-6 pixels depending on layer                          │
│    - Fill color: Activation intensity (0=dim green, 1=white)               │
│    - Border: 1px bright green                                               │
│    - Label: Input nodes show "Bx" "By" "Vx" "Vy" "Py" "Oy"                │
│                                                                             │
│    EDGES (Weights):                                                         │
│    - Line thickness: |weight| mapped to 1-4 pixels                         │
│    - Color: Positive = green, Negative = red                               │
│    - Opacity: |weight| (small weights fade out)                            │
│    - Animation: Pulse on gradient update                                    │
│                                                                             │
│  LAYOUT ALGORITHM:                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐    │
│  │                                                                    │    │
│  │   INPUT        HIDDEN 1       HIDDEN 2       OUTPUT               │    │
│  │                                                                    │    │
│  │     ○            ○              ○              ○  ← UP             │    │
│  │     ○            ○              ○                                  │    │
│  │     ○            ○              ○              ○  ← STAY           │    │
│  │     ○            ○              ○                                  │    │
│  │     ○            ○              ○              ○  ← DOWN           │    │
│  │     ○            ○              ○                                  │    │
│  │                  ○              ○                                  │    │
│  │                  ○                                                 │    │
│  │                                                                    │    │
│  │   x_spacing = panel_width / (num_layers + 1)                      │    │
│  │   y_spacing = panel_height / (max_neurons_in_layer + 1)           │    │
│  │                                                                    │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  TIER-SPECIFIC ADAPTATIONS:                                                 │
│                                                                             │
│  Tier 0-1 (≤64 hidden neurons):                                            │
│    - Show ALL weights as individual lines                                   │
│    - Full detail view                                                       │
│                                                                             │
│  Tier 2 (65-128 hidden neurons):                                           │
│    - Aggregate: Show every 2nd neuron                                       │
│    - Or: Heat map overlay instead of individual lines                      │
│                                                                             │
│  Tier 3-4 (>128 hidden neurons):                                           │
│    - Layer summary view: boxes with aggregate stats                        │
│    - Mean/std of weights per layer                                         │
│    - Activation histogram per layer                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Weight Animation System

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      WEIGHT CHANGE VISUALIZATION                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  GOAL: Show weights "morphing" over time, not just static state            │
│                                                                             │
│  TECHNIQUE 1: Interpolated Drawing                                          │
│  ─────────────────────────────────────────                                  │
│  - Store previous weights (W_prev) and current weights (W_curr)            │
│  - Each frame, draw: W_display = lerp(W_prev, W_curr, t)                   │
│  - t increments from 0→1 over ~10 frames after each update                 │
│  - Creates smooth transition effect                                         │
│                                                                             │
│  TECHNIQUE 2: Gradient Flash                                                │
│  ─────────────────────────────────────────                                  │
│  - On backprop, record which weights changed significantly                  │
│  - Flash those connections bright white, then fade to normal               │
│  - Shows "where learning is happening"                                      │
│                                                                             │
│  TECHNIQUE 3: Historical Trails                                             │
│  ─────────────────────────────────────────                                  │
│  - Store last N values of each weight                                       │
│  - Draw faint "ghost" lines showing where weight used to be                │
│  - Memory intensive - only for Tier 0-1                                    │
│                                                                             │
│  IMPLEMENTATION:                                                            │
│  ```c                                                                       │
│  typedef struct {                                                           │
│      float current;                                                         │
│      float previous;                                                        │
│      float gradient;        // Last gradient magnitude                      │
│      uint8_t flash_timer;   // Countdown for flash effect                  │
│  } WeightVis;                                                               │
│                                                                             │
│  void update_weight_vis(WeightVis* w, float new_val, float grad) {         │
│      w->previous = w->current;                                              │
│      w->current = new_val;                                                  │
│      w->gradient = grad;                                                    │
│      if (fabsf(grad) > GRAD_FLASH_THRESHOLD) {                             │
│          w->flash_timer = FLASH_DURATION;                                  │
│      }                                                                      │
│  }                                                                          │
│  ```                                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Loss Curve Rendering

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         LOSS CURVE VISUALIZATION                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  DATA STRUCTURE:                                                            │
│  ```c                                                                       │
│  #define LOSS_HISTORY_SIZE 1000                                            │
│  typedef struct {                                                           │
│      float values[LOSS_HISTORY_SIZE];                                      │
│      int head;              // Ring buffer head                            │
│      int count;             // Number of valid entries                     │
│      float min_val, max_val; // For auto-scaling Y axis                    │
│      float smoothed;        // EMA for display                             │
│  } LossHistory;                                                             │
│  ```                                                                        │
│                                                                             │
│  RENDERING (320px wide panel):                                              │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ LOSS                                                          [AUTO] │  │
│  │ 2.5├╮                                                                │  │
│  │    │ ╲                                                               │  │
│  │ 1.5├  ╲╮                                                             │  │
│  │    │   ╲__                                                           │  │
│  │ 0.5├      ╲____╱╲_____                                               │  │
│  │    │                  ╲_________________________________▪            │  │
│  │ 0.0├─────────────────────────────────────────────────────────────▶  │  │
│  │    0                                                          1000  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  FEATURES:                                                                  │
│  - Auto-scaling Y axis with min/max tracking                               │
│  - Smooth (EMA) vs Raw toggle                                               │
│  - Current value displayed as pulsing dot at end                           │
│  - Grid lines at meaningful intervals                                       │
│  - "Milestone" markers (vertical lines) at significant events              │
│                                                                             │
│  ALGORITHM:                                                                 │
│  ```c                                                                       │
│  void render_loss_curve(LossHistory* h, int x, int y, int w, int h) {      │
│      // Auto-scale Y axis                                                   │
│      float y_range = h->max_val - h->min_val;                              │
│      if (y_range < 0.1f) y_range = 0.1f; // Minimum range                  │
│                                                                             │
│      // Draw grid                                                           │
│      for (int i = 0; i <= 4; i++) {                                        │
│          int grid_y = y + h - (i * h / 4);                                 │
│          draw_hline(x, grid_y, w, PHOSPHOR_DIM);                           │
│      }                                                                      │
│                                                                             │
│      // Plot points                                                         │
│      int prev_px = -1, prev_py = -1;                                       │
│      for (int i = 0; i < h->count; i++) {                                  │
│          int idx = (h->head - h->count + i + LOSS_HISTORY_SIZE)            │
│                    % LOSS_HISTORY_SIZE;                                    │
│          float val = h->values[idx];                                       │
│                                                                             │
│          int px = x + (i * w / h->count);                                  │
│          int py = y + h - (int)((val - h->min_val) / y_range * h);        │
│                                                                             │
│          if (prev_px >= 0) {                                               │
│              draw_line(prev_px, prev_py, px, py, TERMINAL_GREEN);          │
│          }                                                                  │
│          prev_px = px; prev_py = py;                                       │
│      }                                                                      │
│  }                                                                          │
│  ```                                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.4 Weight Distribution Histogram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       WEIGHT HISTOGRAM VISUALIZATION                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PURPOSE: Show distribution of weight values across network                 │
│           Educational: "Weights start random, converge to useful patterns" │
│                                                                             │
│  DISPLAY:                                                                   │
│  ┌────────────────────────────────────┐                                    │
│  │                ▄                   │                                    │
│  │               ▄█▄                  │                                    │
│  │              ▄███▄                 │                                    │
│  │            ▂▄█████▄▂               │                                    │
│  │          ▂▄███████████▄▂           │                                    │
│  │▁▁▁▁▁▁▁▁▂▄███████████████▄▂▁▁▁▁▁▁▁▁│                                    │
│  ├────────────────────────────────────┤                                    │
│  │ -2.0    -1.0    0.0    +1.0   +2.0 │                                    │
│  │            WEIGHT VALUE            │                                    │
│  │                                    │                                    │
│  │ μ=-0.02  σ=0.45  [LAYER: ALL]     │                                    │
│  └────────────────────────────────────┘                                    │
│                                                                             │
│  BINS: 32 bins covering -3.0 to +3.0 (weights rarely exceed this)          │
│  HEIGHT: Normalized to max bin count                                        │
│  STATISTICS: Mean (μ), Std Dev (σ) displayed below                         │
│                                                                             │
│  ANIMATION:                                                                 │
│  - Update histogram every N training steps (not every frame)               │
│  - Smooth transitions between states                                        │
│  - Show "before/after" comparison on reset                                  │
│                                                                             │
│  LAYER SELECTOR:                                                            │
│  - D-pad left/right cycles through: ALL → L1 → L2 → L3 → OUTPUT → ALL     │
│  - Shows per-layer distribution                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.5 Activation Flow Visualization

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    REAL-TIME ACTIVATION FLOW                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  CONCEPT: Show data "flowing" through network each forward pass             │
│                                                                             │
│  VISUAL METAPHOR: "Electricity flowing through circuit"                     │
│                                                                             │
│  INPUT LAYER:              HIDDEN LAYERS:           OUTPUT:                 │
│  ┌─────────────┐          ┌─────────────┐         ┌─────────────┐          │
│  │ Ball X: ███ │──▶       │ ░░▓▓████░░░ │──▶      │ UP:   ░░░░  │          │
│  │ Ball Y: ██░ │──▶       │ ▓▓▓▓▓▓▓▓▓▓▓ │──▶      │ STAY: ████  │ ← CHOSEN │
│  │ Vel X:  ░██ │──▶       │ ░░░░████░░░ │──▶      │ DOWN: ██░░  │          │
│  │ Vel Y:  ██░ │──▶       │ ░▓▓▓▓▓▓▓▓░░ │──▶      └─────────────┘          │
│  │ Pad Y:  ███ │──▶       │ ▓▓▓▓░░░▓▓▓▓ │──▶                               │
│  │ Opp Y:  ░░█ │──▶       │ ░░░░░░░░░░░ │──▶                               │
│  └─────────────┘          └─────────────┘                                   │
│                                                                             │
│  RENDERING TECHNIQUE:                                                       │
│                                                                             │
│  1. After each forward pass, cache all layer activations                   │
│  2. Draw neurons as circles with fill = activation intensity               │
│  3. Animate "pulses" traveling along high-weight connections               │
│  4. Output layer shows softmax probabilities as bar chart                  │
│                                                                             │
│  PULSE ANIMATION:                                                           │
│  ```c                                                                       │
│  // For each active connection                                              │
│  typedef struct {                                                           │
│      int from_layer, from_neuron;                                          │
│      int to_layer, to_neuron;                                              │
│      float progress;  // 0.0 to 1.0                                        │
│      float intensity; // Brightness of pulse                               │
│  } Pulse;                                                                   │
│                                                                             │
│  // Spawn pulses for high-activation paths                                 │
│  if (activation > PULSE_THRESHOLD && weight > WEIGHT_THRESHOLD) {          │
│      spawn_pulse(layer, neuron, next_layer, next_neuron,                   │
│                  activation * fabsf(weight));                              │
│  }                                                                          │
│  ```                                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.6 Gradient Flow Visualization (Advanced)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    BACKPROPAGATION VISUALIZATION                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PURPOSE: Show gradient flowing BACKWARD through network during training    │
│           "Watch the network learn from its mistakes"                       │
│                                                                             │
│  TRIGGER: Enable in settings, shows during training update step            │
│                                                                             │
│  VISUAL:                                                                    │
│  ┌────────────────────────────────────────────────────────────────────┐    │
│  │                                                                    │    │
│  │  OUTPUT ◄────────── HIDDEN 2 ◄────────── HIDDEN 1 ◄────── INPUT  │    │
│  │                                                                    │    │
│  │    ◉ ◄════════════════╗                                           │    │
│  │    ○ ◄═══════════════╬════════════╗                               │    │
│  │    ◉ ◄══════════════╬╬═══════════╬════════════╗                   │    │
│  │                      ╚╬═══════════╬════════════╬═══════ ○         │    │
│  │                       ╚═══════════╬════════════╬═══════ ○         │    │
│  │                                   ╚════════════╬═══════ ◉         │    │
│  │                                                ╚═══════ ○         │    │
│  │                                                                    │    │
│  │  [GRADIENT MAGNITUDE: ════ large  ──── medium  .... small]        │    │
│  │                                                                    │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  COLORS:                                                                    │
│  - AMBER: Gradient flow (distinct from green forward pass)                 │
│  - Brightness: Gradient magnitude                                           │
│  - Animation: Pulses travel right-to-left (backward)                       │
│                                                                             │
│  EDUCATIONAL CALLOUTS:                                                      │
│  - "Large gradient = big weight update"                                    │
│  - "Gradient shrinking = vanishing gradient problem"                       │
│  - Can toggle between forward/backward view modes                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. USER INTERFACE SCREENS

### 7.1 Title Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                                                                             │
│              ███╗   ██╗███████╗██╗   ██╗██████╗  ██████╗ ███╗   ██╗        │
│              ████╗  ██║██╔════╝██║   ██║██╔══██╗██╔═══██╗████╗  ██║        │
│              ██╔██╗ ██║█████╗  ██║   ██║██████╔╝██║   ██║██╔██╗ ██║        │
│              ██║╚██╗██║██╔══╝  ██║   ██║██╔══██╗██║   ██║██║╚██╗██║        │
│              ██║ ╚████║███████╗╚██████╔╝██║  ██║╚██████╔╝██║ ╚████║        │
│              ╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝        │
│                                                                             │
│                        NEURAL NETWORK LABORATORY                            │
│                              ══════════════                                 │
│                                                                             │
│                    ┌─────────────────────────────────┐                      │
│                    │  ░░░░░░░ LOADING ░░░░░░░░░░░░░ │                      │
│                    │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░ │                      │
│                    └─────────────────────────────────┘                      │
│                                                                             │
│                                                                             │
│                         - - PRESS START - -                                │
│                                                                             │
│                                                                             │
│                                                                             │
│ ─────────────────────────────────────────────────────────────────────────  │
│ BLUE FROG ANALYTICS // DEEP PADDLE v1.0 // PLATFORM: N64 // RAM: 4MB      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Main Menu

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ SYSTEM ──────────────────────────────────────────────────────────────┐  │
│ │ NEURON v1.0 ░░ MAIN MENU ░░ [CONTROLLER 1 ACTIVE]                    │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│    ┌─────────────────────────────────────────────────────────────────┐     │
│    │                                                                 │     │
│    │                     M A I N   M E N U                           │     │
│    │                     ═══════════════════                         │     │
│    │                                                                 │     │
│    │           ╔═══════════════════════════════════╗                │     │
│    │           ║ ▶  NEW TRAINING SESSION           ║  ◄────────────│     │
│    │           ╚═══════════════════════════════════╝                │     │
│    │              ┌───────────────────────────────┐                 │     │
│    │              │    LOAD CHECKPOINT            │                 │     │
│    │              └───────────────────────────────┘                 │     │
│    │              ┌───────────────────────────────┐                 │     │
│    │              │    SETTINGS                   │                 │     │
│    │              └───────────────────────────────┘                 │     │
│    │              ┌───────────────────────────────┐                 │     │
│    │              │    HOW IT WORKS               │                 │     │
│    │              └───────────────────────────────┘                 │     │
│    │              ┌───────────────────────────────┐                 │     │
│    │              │    ABOUT                      │                 │     │
│    │              └───────────────────────────────┘                 │     │
│    │                                                                 │     │
│    └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ ─────────────────────────────────────────────────────────────────────────  │
│ [A] SELECT    [B] BACK    [START] BEGIN                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.3 Settings Panel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ SETTINGS ────────────────────────────────────────────────────────────┐  │
│ │ CONFIGURATION PANEL ░░ NETWORK & TRAINING PARAMETERS                 │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─ NETWORK ARCHITECTURE ──────────────────┐  ┌─ HYPERPARAMETERS ───────┐  │
│  │                                          │  │                         │  │
│  │  COMPLEXITY TIER:                        │  │  LEARNING RATE (η)      │  │
│  │  ┌────────────────────────────────────┐ │  │  ◄ 0.00100 ▶            │  │
│  │  │ ░░ MINIMAL ░░ LIGHT ▓▓ MEDIUM ░░  │ │  │  ──────●────────────    │  │
│  │  │ ░░ HEAVY ░░░░ SUPERHEAVY ░░░░░░░░ │ │  │  0.0001          0.01   │  │
│  │  └────────────────────────────────────┘ │  │                         │  │
│  │                                          │  │  EPSILON DECAY          │  │
│  │  SELECTED: LIGHT                         │  │  ◄ 0.9995 ▶            │  │
│  │  ├─ Layers: 6 → 32 → 32 → 3             │  │  ──────────●──────────  │  │
│  │  ├─ Parameters: 1,379                   │  │  0.999        0.99999   │  │
│  │  ├─ Memory: ~16KB                       │  │                         │  │
│  │  └─ Est. Time to Competent: 30 min      │  │  DISCOUNT (γ)           │  │
│  │                                          │  │  ◄ 0.99 ▶              │  │
│  └──────────────────────────────────────────┘  │  ───────────────●────  │  │
│                                                 │  0.9              0.999│  │
│  ┌─ TRAINING OPTIONS ───────────────────────┐  │                         │  │
│  │                                           │  │  BATCH SIZE             │  │
│  │  [●] Experience Replay Enabled            │  │  ◄ 32 ▶                │  │
│  │  [ ] Double DQN (advanced)                │  │  [ 16 | 32 | 64 ]      │  │
│  │  [●] Reward Shaping Enabled               │  │                         │  │
│  │  [ ] Gradient Clipping                    │  └─────────────────────────┘  │
│  │                                           │                              │
│  │  SPEED MULTIPLIER:                        │  ┌─ DISPLAY OPTIONS ──────┐  │
│  │  [ 1x | 2x | 4x | 8x | MAX ]             │  │ [●] Show Network Graph  │  │
│  │                                           │  │ [●] Show Loss Curve     │  │
│  └───────────────────────────────────────────┘  │ [●] Show Histogram      │  │
│                                                 │ [ ] Show Gradients      │  │
│  ┌─ PRESETS ─────────────────────────────────┐ │ [●] Scan Lines          │  │
│  │ [FAST] [BALANCED] [CAREFUL] [CUSTOM]      │ └─────────────────────────┘  │
│  └───────────────────────────────────────────┘                              │
│ ─────────────────────────────────────────────────────────────────────────── │
│ [A] ADJUST    [B] BACK    [START] CONFIRM & BEGIN                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.4 Main Simulation Screen (Full Layout)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ STATUS ──────────────────────────────────────────────────────────────┐  │
│ │ [▶ TRAINING] EP:00847 │ ε:0.15 │ η:0.001 │ ████░░░░ 4x │ 14:32:07    │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─ GAME ──────────────────────────┐ ┌─ NETWORK ────────────────────────┐  │
│  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│ │ INPUT     HIDDEN      OUTPUT    │  │
│  │▓                              ▓│ │                                   │  │
│  │▓     ○                        ▓│ │  Bx ○─┬───○───┬───○ UP           │  │
│  │▓    /                    ███  ▓│ │  By ○─┼───○───┼                   │  │
│  │▓   /                     ███  ▓│ │  Vx ○─┼───○───┼───● STAY ◄──     │  │
│  │▓  ●                      ███  ▓│ │  Vy ○─┼───○───┼                   │  │
│  │▓                              ▓│ │  Py ○─┼───○───┼───○ DOWN         │  │
│  │▓ ███                          ▓│ │  Oy ○─┴───○───┘                   │  │
│  │▓ ███                          ▓│ │       └───○                       │  │
│  │▓ ███                          ▓│ │           └───○                   │  │
│  │▓                              ▓│ │                                   │  │
│  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│ │ ──positive ──negative            │  │
│  │     AI: 21        OPP: 3      │ │ brightness = activation           │  │
│  └────────────────────────────────┘ └───────────────────────────────────┘  │
│                                                                             │
│  ┌─ LOSS ────────────────────────────────────────────────────────────────┐ │
│  │ 2.0├╮                                                          [RAW] │ │
│  │    │ ╲                                                               │ │
│  │ 1.0├  ╲___                                                           │ │
│  │    │      ╲____╱╲_______________________________________________▪    │ │
│  │ 0.0├─────────────────────────────────────────────────────────────▶  │ │
│  │    0              EPISODE              500                     1000  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─ METRICS ─────────────┐ ┌─ WEIGHTS ───────────┐ ┌─ ACTIONS ──────────┐  │
│  │ WIN RATE    87.3%     │ │      ▁▃▅█▇▅▃▁       │ │ A: Speed   Z: Play │  │
│  │ AVG RALLY   12.4      │ │   -1    0    +1     │ │ B: Speed   START:  │  │
│  │ REWARD/EP   +0.73     │ │ μ:-0.02  σ:0.41     │ │     ↓      Menu    │  │
│  │ STEPS       847,392   │ │ [LAYER: ALL ◄ ►]   │ │ L/R: Histogram     │  │
│  └────────────────────────┘ └────────────────────┘ └─────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.5 Pause Menu Overlay

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░┌────────────────────────────────────┐░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│                                    │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│     ║ ▓ ▓   P A U S E D   ▓ ▓ ║    │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│     ╚═════════════════════════╝    │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│                                    │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│      ╔════════════════════╗       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│      ║ ▶ RESUME           ║       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│      ╚════════════════════╝       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        ┌──────────────────┐       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        │   SAVE STATE     │       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        └──────────────────┘       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        ┌──────────────────┐       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        │   SETTINGS       │       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        └──────────────────┘       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        ┌──────────────────┐       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        │   RESET WEIGHTS  │       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        └──────────────────┘       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        ┌──────────────────┐       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        │   EXIT TO MENU   │       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│        └──────────────────┘       │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│                                    │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░│  EP: 847   TIME: 00:14:32         │░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░└────────────────────────────────────┘░░░░░░░░░░░░░░░     │
│      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.6 "How It Works" Tutorial Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ TUTORIAL ────────────────────────────────────────────────────────────┐  │
│ │ HOW NEURAL NETWORKS LEARN ░░ PAGE 1/5 ░░ FORWARD PASS                │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                                                                       │ │
│  │   THE FORWARD PASS: From Input to Decision                           │ │
│  │   ════════════════════════════════════════                           │ │
│  │                                                                       │ │
│  │   1. INPUTS: The network sees 6 numbers:                             │ │
│  │      Ball position (X,Y), Ball velocity (VX,VY),                     │ │
│  │      Your paddle (Y), Opponent paddle (Y)                            │ │
│  │                                                                       │ │
│  │      ○ Ball X   ═══╗                                                 │ │
│  │      ○ Ball Y   ═══╬═══► HIDDEN ═══► HIDDEN ═══► ○ UP               │ │
│  │      ○ Vel X    ═══╬═══► LAYER  ═══► LAYER  ═══► ● STAY ◄ CHOSEN   │ │
│  │      ○ Vel Y    ═══╬═══►   1    ═══►   2    ═══► ○ DOWN             │ │
│  │      ○ Paddle Y ═══╬                                                 │ │
│  │      ○ Opp Y    ═══╝                                                 │ │
│  │                                                                       │ │
│  │   2. WEIGHTS: Each connection has a "strength" (weight)              │ │
│  │      Strong weights = thick lines = important connections           │ │
│  │                                                                       │ │
│  │   3. OUTPUT: Network outputs confidence for each action              │ │
│  │      Highest confidence wins → paddle moves                          │ │
│  │                                                                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│ ─────────────────────────────────────────────────────────────────────────  │
│ [◄ PREV]                  ● ○ ○ ○ ○                          [NEXT ►]     │
│ [B] BACK TO MENU                                                           │
└─────────────────────────────────────────────────────────────────────────────┘

Tutorial Pages:
  1. Forward Pass (above)
  2. The Loss Function ("How wrong was the prediction?")
  3. Backpropagation ("Adjusting weights to reduce error")
  4. Exploration vs Exploitation ("Random moves help learning")
  5. Watch It Learn ("Let's see it in action!")
```

---

## 8. DATA PERSISTENCE

### 8.1 Save System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SAVE DATA STRUCTURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  CONTROLLER PAK MEMORY MAP (32KB total, 123 pages × 256 bytes):            │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Page 0-3 (1KB): HEADER & METADATA                                   │   │
│  │ ├── Magic number: "NRNN" (4 bytes)                                  │   │
│  │ ├── Version: 1.0 (2 bytes)                                          │   │
│  │ ├── Network tier: 0-4 (1 byte)                                      │   │
│  │ ├── Episode count (4 bytes)                                         │   │
│  │ ├── Total steps (4 bytes)                                           │   │
│  │ ├── Best win rate (4 bytes, float)                                  │   │
│  │ ├── Hyperparameters snapshot (32 bytes)                             │   │
│  │ ├── Training time (4 bytes, seconds)                                │   │
│  │ └── Checksum (4 bytes)                                              │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │ Page 4-67 (16KB): NETWORK WEIGHTS (Tier 1)                         │   │
│  │ ├── Layer 1 weights: 6×32 = 192 floats = 768 bytes                 │   │
│  │ ├── Layer 1 biases: 32 floats = 128 bytes                          │   │
│  │ ├── Layer 2 weights: 32×32 = 1024 floats = 4096 bytes              │   │
│  │ ├── Layer 2 biases: 32 floats = 128 bytes                          │   │
│  │ ├── Output weights: 32×3 = 96 floats = 384 bytes                   │   │
│  │ └── Output biases: 3 floats = 12 bytes                             │   │
│  │     TOTAL: ~5.5KB for Tier 1                                        │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │ Page 68-83 (4KB): TRAINING STATE                                   │   │
│  │ ├── Current epsilon (4 bytes)                                       │   │
│  │ ├── Adam optimizer momentum (if enabled)                            │   │
│  │ └── Recent loss history (256 floats = 1KB)                         │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │ Page 84-123 (10KB): STATISTICS HISTORY                              │   │
│  │ ├── Win rate per 100 episodes (variable)                           │   │
│  │ └── Milestone markers                                               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  NOTE: Tier 2+ networks may not fully fit in Controller Pak.               │
│        Options: (a) Save only weights, reinit optimizer state              │
│                 (b) Compress weights (quantize to 8-bit)                   │
│                 (c) Multiple save slots for partial saves                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Built-in Checkpoints (ROM)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PRE-TRAINED CHECKPOINTS                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Bundled in ROM for instant demonstration:                                  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CHECKPOINT 0: "RANDOM"                                              │   │
│  │ - Freshly initialized random weights                                │   │
│  │ - Xavier/He initialization                                          │   │
│  │ - Win rate: ~15% (random chance)                                    │   │
│  │ - Use case: "Watch learning from scratch"                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CHECKPOINT 1: "NOVICE" (~500 episodes)                              │   │
│  │ - Basic ball tracking behavior                                      │   │
│  │ - Still makes obvious mistakes                                      │   │
│  │ - Win rate: ~45%                                                    │   │
│  │ - Use case: "See early learning progress"                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CHECKPOINT 2: "COMPETENT" (~2000 episodes)                          │   │
│  │ - Solid positioning, intercepts most shots                         │   │
│  │ - Occasional errors on edge cases                                   │   │
│  │ - Win rate: ~75%                                                    │   │
│  │ - Use case: "Fun to play against"                                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CHECKPOINT 3: "EXPERT" (~10000 episodes)                            │   │
│  │ - Near-optimal play                                                 │   │
│  │ - Predicts trajectories accurately                                  │   │
│  │ - Win rate: ~95%                                                    │   │
│  │ - Use case: "Demonstration of mastery"                             │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 9. TECHNICAL IMPLEMENTATION

### 9.1 Development Environment

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DEVELOPMENT STACK                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  TOOLCHAIN:                                                                 │
│  ├── Language: C (C99 standard)                                            │
│  ├── SDK: libdragon (open-source N64 SDK)                                  │
│  │         https://github.com/DragonMinded/libdragon                       │
│  ├── Compiler: GCC MIPS cross-compiler                                     │
│  ├── Build: Make                                                           │
│  └── Emulator: Ares, simple64, or Project64 for testing                   │
│                                                                             │
│  LIBDRAGON FEATURES USED:                                                   │
│  ├── display.h - Framebuffer management                                    │
│  ├── graphics.h - Primitive drawing (lines, rects, text)                   │
│  ├── controller.h - Input handling                                         │
│  ├── timer.h - Frame timing, profiling                                     │
│  ├── debug.h - printf to emulator console                                  │
│  └── dfs.h - DragonFS for asset loading                                    │
│                                                                             │
│  REPOSITORY STRUCTURE:                                                      │
│  ```                                                                        │
│  neuron-n64/                                                                │
│  ├── src/                                                                   │
│  │   ├── main.c              # Entry point, state machine                  │
│  │   ├── neural_net.c        # Forward/backward pass, weight update        │
│  │   ├── neural_net.h                                                      │
│  │   ├── training.c          # DQN algorithm, replay buffer               │
│  │   ├── training.h                                                        │
│  │   ├── pong.c              # Game simulation                             │
│  │   ├── pong.h                                                            │
│  │   ├── render.c            # All visualization                           │
│  │   ├── render.h                                                          │
│  │   ├── ui.c                # Menus, settings panels                      │
│  │   ├── ui.h                                                              │
│  │   ├── save.c              # Controller Pak I/O                          │
│  │   └── save.h                                                            │
│  ├── assets/                                                                │
│  │   ├── fonts/              # Custom bitmap fonts                         │
│  │   ├── sprites/            # UI elements                                 │
│  │   └── checkpoints/        # Pre-trained weight files                   │
│  ├── include/                                                               │
│  │   └── config.h            # Build-time configuration                    │
│  ├── Makefile                                                               │
│  └── README.md                                                              │
│  ```                                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.2 Core Data Structures

```c
// config.h - Build-time configuration
#define MAX_LAYERS 6
#define MAX_NEURONS_PER_LAYER 256
#define REPLAY_BUFFER_SIZE 2000
#define LOSS_HISTORY_SIZE 1000

// neural_net.h
typedef struct {
    int num_layers;
    int layer_sizes[MAX_LAYERS];
    
    // Weights: weights[layer][to][from]
    float* weights[MAX_LAYERS];  // Flattened 2D arrays
    float* biases[MAX_LAYERS];
    
    // For training
    float* weight_grads[MAX_LAYERS];
    float* bias_grads[MAX_LAYERS];
    
    // Adam optimizer state (optional)
    float* weight_m[MAX_LAYERS];  // First moment
    float* weight_v[MAX_LAYERS];  // Second moment
    
    // Activations cache (for backprop)
    float* activations[MAX_LAYERS];
    float* pre_activations[MAX_LAYERS];  // Before ReLU
    
    // Visualization state
    float* prev_weights[MAX_LAYERS];     // For animation
    uint8_t* weight_flash[MAX_LAYERS];   // Flash timers
} NeuralNetwork;

// training.h
typedef struct {
    float state[6];
    int action;
    float reward;
    float next_state[6];
    uint8_t done;
} Transition;

typedef struct {
    Transition buffer[REPLAY_BUFFER_SIZE];
    int head;
    int count;
} ReplayBuffer;

typedef struct {
    float learning_rate;
    float gamma;           // Discount factor
    float epsilon;         // Exploration rate
    float epsilon_min;
    float epsilon_decay;
    int batch_size;
    int target_update_freq;
    
    // Statistics
    int total_episodes;
    int total_steps;
    float loss_history[LOSS_HISTORY_SIZE];
    int loss_head;
    float win_rate;
    float avg_reward;
} TrainingState;

// pong.h
typedef struct {
    float ball_x, ball_y;
    float ball_vx, ball_vy;
    float ai_paddle_y;
    float opp_paddle_y;
    int ai_score, opp_score;
    uint8_t ball_served;
} PongState;

// render.h
typedef struct {
    int show_network;
    int show_loss_curve;
    int show_histogram;
    int show_gradients;
    int histogram_layer;  // -1 for all
    int animation_speed;
    uint8_t scanlines_enabled;
} RenderSettings;
```

### 9.3 Core Algorithm Pseudocode

```c
// main.c - Main loop
void main_loop(void) {
    while (1) {
        controller_scan();
        
        switch (app_state) {
            case STATE_SIM_TRAINING:
                // Run N game steps per frame (speed multiplier)
                for (int i = 0; i < speed_multiplier; i++) {
                    // 1. Get state
                    float state[6];
                    get_normalized_state(&pong, state);
                    
                    // 2. Select action (epsilon-greedy)
                    int action;
                    if (randf() < training.epsilon) {
                        action = rand() % 3;
                    } else {
                        action = nn_get_best_action(&network, state);
                    }
                    
                    // 3. Execute action
                    execute_action(&pong, action);
                    pong_step(&pong);
                    
                    // 4. Get reward and new state
                    float reward = calculate_reward(&pong);
                    float next_state[6];
                    get_normalized_state(&pong, next_state);
                    uint8_t done = pong.ball_served == 0;
                    
                    // 5. Store transition
                    replay_buffer_add(&buffer, state, action, reward, 
                                     next_state, done);
                    
                    // 6. Train on batch
                    if (buffer.count >= training.batch_size) {
                        float loss = train_batch(&network, &buffer, &training);
                        loss_history_add(&training, loss);
                    }
                    
                    // 7. Decay epsilon
                    if (training.epsilon > training.epsilon_min) {
                        training.epsilon *= training.epsilon_decay;
                    }
                    
                    // 8. Handle episode end
                    if (done) {
                        training.total_episodes++;
                        update_statistics(&training, &pong);
                        pong_reset(&pong);
                    }
                    
                    training.total_steps++;
                }
                break;
                
            case STATE_SIM_PLAY:
                // Human controls AI paddle, network controls opponent
                // ... (no training, just inference)
                break;
        }
        
        // Render frame
        render_frame(&pong, &network, &training, &render_settings);
        
        // Handle input
        handle_input();
    }
}

// neural_net.c - Forward pass
void nn_forward(NeuralNetwork* nn, float* input, float* output) {
    // Copy input to first activation layer
    memcpy(nn->activations[0], input, nn->layer_sizes[0] * sizeof(float));
    
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l-1];
        int curr_size = nn->layer_sizes[l];
        
        for (int j = 0; j < curr_size; j++) {
            float sum = nn->biases[l][j];
            for (int i = 0; i < prev_size; i++) {
                sum += nn->weights[l][j * prev_size + i] * nn->activations[l-1][i];
            }
            nn->pre_activations[l][j] = sum;
            
            // ReLU for hidden layers, linear for output
            if (l < nn->num_layers - 1) {
                nn->activations[l][j] = (sum > 0) ? sum : 0;
            } else {
                nn->activations[l][j] = sum;
            }
        }
    }
    
    // Copy output
    memcpy(output, nn->activations[nn->num_layers - 1], 
           nn->layer_sizes[nn->num_layers - 1] * sizeof(float));
}

// training.c - Backpropagation
float train_batch(NeuralNetwork* nn, ReplayBuffer* buf, TrainingState* ts) {
    float total_loss = 0;
    
    // Sample random batch
    for (int b = 0; b < ts->batch_size; b++) {
        int idx = rand() % buf->count;
        Transition* t = &buf->buffer[idx];
        
        // Forward pass
        float q_values[3];
        nn_forward(nn, t->state, q_values);
        
        // Compute target
        float next_q[3];
        nn_forward(nn, t->next_state, next_q);
        float max_next_q = fmaxf(fmaxf(next_q[0], next_q[1]), next_q[2]);
        float target = t->reward + (t->done ? 0 : ts->gamma * max_next_q);
        
        // Compute loss (MSE on taken action only)
        float error = target - q_values[t->action];
        total_loss += error * error;
        
        // Backpropagate
        nn_backward(nn, t->action, error);
    }
    
    // Update weights
    nn_update_weights(nn, ts->learning_rate);
    
    return total_loss / ts->batch_size;
}
```

---

## 10. PROJECT TIMELINE

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DEVELOPMENT PHASES                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PHASE 1: FOUNDATION (Week 1-2)                                            │
│  ══════════════════════════════                                            │
│  □ Set up libdragon development environment                                │
│  □ Basic N64 "hello world" - framebuffer, text, input                     │
│  □ Implement Pong game logic (no AI)                                       │
│  □ Human vs Human playable                                                 │
│  □ Basic UI framework (menus, panels)                                      │
│                                                                             │
│  PHASE 2: NEURAL NETWORK ENGINE (Week 3-4)                                 │
│  ═════════════════════════════════════════                                 │
│  □ Implement forward pass (arbitrary layer sizes)                          │
│  □ Implement backpropagation                                               │
│  □ Implement weight update (SGD, optionally Adam)                          │
│  □ Unit tests on PC before N64 port                                        │
│  □ Benchmark inference speed on N64                                        │
│                                                                             │
│  PHASE 3: TRAINING LOOP (Week 5-6)                                         │
│  ═════════════════════════════════                                         │
│  □ Implement DQN algorithm                                                 │
│  □ Implement replay buffer                                                 │
│  □ State normalization                                                     │
│  □ Reward shaping                                                          │
│  □ Verify learning works (agent improves over time)                        │
│  □ Tune hyperparameters for ~1 hour learning curve                         │
│                                                                             │
│  PHASE 4: VISUALIZATION (Week 7-9)                                         │
│  ═════════════════════════════════                                         │
│  □ Network topology renderer                                               │
│  □ Weight animation system                                                 │
│  □ Loss curve graphing                                                     │
│  □ Activation visualization                                                │
│  □ Weight histogram                                                        │
│  □ Gradient flow (optional advanced feature)                               │
│  □ Performance optimization (stay at 30fps)                                │
│                                                                             │
│  PHASE 5: POLISH (Week 10-11)                                              │
│  ════════════════════════════                                              │
│  □ UI/UX refinement                                                        │
│  □ Skunkworks visual theme implementation                                  │
│  □ Settings panel                                                          │
│  □ Network tier system                                                     │
│  □ Tutorial/How It Works screens                                           │
│  □ Sound effects (optional)                                                │
│                                                                             │
│  PHASE 6: PERSISTENCE & RELEASE (Week 12)                                  │
│  ════════════════════════════════════════                                  │
│  □ Controller Pak save/load                                                │
│  □ Pre-trained checkpoints in ROM                                          │
│  □ Testing on real hardware                                                │
│  □ Documentation                                                           │
│  □ Release ROM + source code                                               │
│                                                                             │
│  STRETCH GOALS:                                                            │
│  ══════════════                                                            │
│  □ Two-player mode (both humans train separate networks)                   │
│  □ Network architecture editor                                             │
│  □ Export weights to file (via Transfer Pak or custom hardware)           │
│  □ Alternative games (Breakout, simple platformer)                         │
│  □ Comparison mode (run two algorithms side-by-side)                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 11. EDUCATIONAL IMPACT

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      LEARNING OBJECTIVES                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  VIEWERS WILL UNDERSTAND:                                                   │
│                                                                             │
│  1. WHAT A NEURAL NETWORK IS                                               │
│     "Layers of numbers connected by weighted edges"                         │
│     Visual: See the topology, watch data flow through                      │
│                                                                             │
│  2. WHAT WEIGHTS DO                                                         │
│     "Connections that strengthen or weaken based on experience"            │
│     Visual: Watch line thickness change over time                          │
│                                                                             │
│  3. WHAT A FORWARD PASS IS                                                 │
│     "Input goes in, decision comes out"                                    │
│     Visual: See activations light up, output selected                      │
│                                                                             │
│  4. WHAT LOSS/ERROR MEANS                                                  │
│     "How wrong the network was"                                            │
│     Visual: Loss curve descending over time                                │
│                                                                             │
│  5. WHAT BACKPROPAGATION DOES                                              │
│     "Adjusting weights to reduce error"                                    │
│     Visual: Gradient flow animation (advanced mode)                        │
│                                                                             │
│  6. WHAT EXPLORATION VS EXPLOITATION IS                                    │
│     "Try random things vs use what you know"                               │
│     Visual: Epsilon value display, random move indicator                   │
│                                                                             │
│  7. THAT AI "LEARNING" IS JUST MATH                                        │
│     "No magic, just multiplication and addition"                           │
│     Visual: Running on 1996 hardware proves simplicity                     │
│                                                                             │
│  8. THAT MORE CAPACITY ≠ FASTER LEARNING                                   │
│     "Bigger networks learn slower but can get better"                      │
│     Visual: Tier comparison, complexity tradeoffs                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 12. RISKS & MITIGATIONS

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RISK ASSESSMENT                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  RISK: N64 FPU too slow for real-time training                            │
│  LIKELIHOOD: Medium                                                         │
│  IMPACT: High                                                               │
│  MITIGATION:                                                                │
│  - Use fixed-point arithmetic if needed                                    │
│  - Reduce batch size                                                        │
│  - Train every N frames instead of every frame                             │
│  - Pre-profile on emulator before hardware                                 │
│                                                                             │
│  RISK: Learning doesn't converge in reasonable time                        │
│  LIKELIHOOD: Medium                                                         │
│  IMPACT: High                                                               │
│  MITIGATION:                                                                │
│  - Extensive hyperparameter tuning on PC first                             │
│  - Reward shaping to accelerate learning                                   │
│  - Simpler opponent AI (predictable, easier to beat)                       │
│  - Pre-trained checkpoints as fallback demonstration                       │
│                                                                             │
│  RISK: Visualization tanks framerate                                       │
│  LIKELIHOOD: High                                                           │
│  IMPACT: Medium                                                             │
│  MITIGATION:                                                                │
│  - Update visualizations every N frames, not every frame                   │
│  - Tier-based visualization complexity                                     │
│  - Toggle individual visualizations on/off                                 │
│  - Use RDP for hardware-accelerated drawing where possible                │
│                                                                             │
│  RISK: Controller Pak too small for larger networks                        │
│  LIKELIHOOD: Certain (for Tier 3+)                                         │
│  IMPACT: Low                                                                │
│  MITIGATION:                                                                │
│  - Quantize weights to 8-bit for storage                                   │
│  - Store only weights, reinit optimizer state on load                      │
│  - Tier 3+ marked as "session only" (no save)                             │
│                                                                             │
│  RISK: Scope creep extends timeline                                        │
│  LIKELIHOOD: High                                                           │
│  IMPACT: Medium                                                             │
│  MITIGATION:                                                                │
│  - Clear MVP definition: Pong + DQN + Basic viz + Tier 0-1                │
│  - Everything else is stretch goal                                         │
│  - Regular milestone check-ins                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 13. SUCCESS METRICS

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEFINITION OF DONE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  MVP COMPLETE WHEN:                                                         │
│  ☑ Network learns to beat random opponent within 1 hour                    │
│  ☑ Network visualization clearly shows weights changing                    │
│  ☑ Loss curve visibly descends during training                            │
│  ☑ Runs at stable 30fps on real N64 hardware                              │
│  ☑ User can play against trained network                                   │
│  ☑ At least 2 network tiers selectable                                     │
│                                                                             │
│  STRETCH COMPLETE WHEN:                                                     │
│  ☑ All 5 network tiers implemented                                         │
│  ☑ Gradient flow visualization working                                     │
│  ☑ Controller Pak save/load working                                        │
│  ☑ Tutorial screens implemented                                            │
│  ☑ Multiple pre-trained checkpoints included                               │
│  ☑ Speed multiplier (1x to MAX) working                                    │
│                                                                             │
│  QUALITATIVE SUCCESS:                                                       │
│  ☑ Non-technical person can understand "the AI is learning"               │
│  ☑ Visual aesthetic matches "skunkworks terminal" brief                   │
│  ☑ Watching training is genuinely engaging (not boring)                   │
│  ☑ "Wow, this runs on an N64" reaction                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## APPENDIX A: REFERENCE IMPLEMENTATIONS

- **sgd.fyi** - Loss/weight visualization inspiration
- **nanoGPT** (Karpathy) - Minimal neural net training
- **OpenAI Gym Pong** - Pong RL environment reference  
- **libdragon examples** - N64 graphics/input patterns
- **3blue1brown neural network videos** - Educational visualization style

---

## APPENDIX B: ESTIMATED MEMORY BUDGET (TIER 1, DEFAULT)

```
┌────────────────────────────────────────────────────────┐
│ COMPONENT                               SIZE           │
├────────────────────────────────────────────────────────┤
│ Network weights (1,379 params × 4)      5.5 KB        │
│ Network gradients (mirror)              5.5 KB        │
│ Adam momentum (m + v)                   11 KB         │
│ Activations cache                       1 KB          │
│ Previous weights (for animation)        5.5 KB        │
│ Flash timers                            1.4 KB        │
├────────────────────────────────────────────────────────┤
│ Replay buffer (2000 × 52 bytes)         104 KB        │
│ Loss history (1000 × 4 bytes)           4 KB          │
│ Episode statistics                      2 KB          │
├────────────────────────────────────────────────────────┤
│ Frame buffers (320×240×2 × 2)           307 KB        │
│ Z-buffer                                154 KB        │
│ UI/Font data                            32 KB         │
│ Code + stack                            256 KB        │
├────────────────────────────────────────────────────────┤
│ TOTAL                                   ~890 KB       │
│ AVAILABLE                               4,096 KB      │
│ HEADROOM                                ~3,200 KB     │
└────────────────────────────────────────────────────────┘

Conclusion: Tier 1 fits comfortably with 75%+ headroom.
            Tier 4 (SUPERHEAVY) would use ~1.8MB, still safe.
```

---

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║                         END OF PROJECT DOCUMENT                              ║
║                                                                              ║
║                    BLUE FROG ANALYTICS // PROJECT NEURON                     ║
║                              CODENAME: DEEP PADDLE                           ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```
