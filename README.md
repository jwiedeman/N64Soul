# PROJECT NEURON
## N64 Neural Network Visualization System
### Codename: DEEP PADDLE

```
+==============================================================================+
|  CLASSIFICATION: EDUCATIONAL / OPEN SOURCE                                   |
|  PLATFORM: Nintendo 64 (4MB RAM / 64MB ROM / 93.75MHz MIPS R4300i)          |
|  OBJECTIVE: Real-time neural network training visualization via Pong RL     |
+==============================================================================+
```

## Mission

Build an interactive, visually rich demonstration of neural network learning that runs on Nintendo 64 hardware. Users observe a neural network learn to play Pong from zero competence to superhuman performance, with full visualization of weights, activations, gradients, and loss curves in real-time.

## Why N64?

- Proves AI fundamentals run on 1996 hardware
- Physical artifact > web demo for memorability
- Constraint breeds creativity in visualization
- "If an N64 can do this, demystify AI" messaging

## Visual Identity: "Skunkworks Terminal"

Cold War signals intelligence meets 90s hacker film aesthetic:
- NORAD war room displays (WarGames, 1983)
- Submarine sonar screens
- Bloomberg terminal density
- Fallout Pip-Boy interface
- Alien (1979) Nostromo computer UI

## Project Structure

```
neuron-n64/
├── src/
│   ├── main.c              # Entry point, state machine
│   ├── neural_net.c        # Forward/backward pass, weight update
│   ├── neural_net.h
│   ├── training.c          # DQN algorithm, replay buffer
│   ├── training.h
│   ├── pong.c              # Game simulation
│   ├── pong.h
│   ├── render.c            # All visualization
│   ├── render.h
│   ├── ui.c                # Menus, settings panels
│   ├── ui.h
│   ├── save.c              # Controller Pak I/O
│   └── save.h
├── assets/
│   ├── fonts/              # Custom bitmap fonts
│   ├── sprites/            # UI elements
│   └── checkpoints/        # Pre-trained weight files
├── include/
│   └── config.h            # Build-time configuration
├── Makefile
└── README.md
```

## Network Architecture Tiers

| Tier | Name | Architecture | Params | Memory | Learn Time |
|------|------|-------------|--------|--------|------------|
| 0 | Minimal | 6→16→3 | 163 | ~2KB | ~15 min |
| 1 | Light | 6→32→32→3 | 1,379 | ~16KB | ~30 min |
| 2 | Medium | 6→64→64→32→3 | ~6,819 | ~80KB | ~1 hour |
| 3 | Heavy | 6→128→128→64→32→3 | ~27,171 | ~320KB | ~3 hours |
| 4 | Superheavy | 6→256→256→128→64→3 | ~107,395 | ~1.2MB | 8+ hours |

## Development

### Prerequisites
- libdragon SDK
- GCC MIPS cross-compiler
- Ares/simple64/Project64 emulator

### Building
```bash
make
```

### Running
```bash
# In emulator
ares neuron.z64
```

## Status

**Phase 1: Foundation** - In Progress

See [PLAN.md](PLAN.md) for detailed project plan and timeline.

## License

Open Source - Educational

---

```
+==============================================================================+
|                    BLUE FROG ANALYTICS // PROJECT NEURON                     |
|                              CODENAME: DEEP PADDLE                           |
+==============================================================================+
```
