# Game Design

This document is the single source of truth for all game mechanics and rules.

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys / D-pad | Move snake / Navigate menu |
| X (Button 1) | Select / Speed Boost (costs 1 energy in battle modes) |
| Z (Button 2) | Back / Slowdown (free, cancels boost) |

## Game Modes

### Classic Mode

Traditional snake rules:
- Collision = instant death
- No length limit (grow indefinitely)
- No energy system (speed control is free)
- No supply packs
- No enemies

### Battle Modes (Noob, Normal, Hell, Nightmare)

Combat-focused gameplay:
- **Collision = Shrink**: Hitting obstacles shrinks snake by 1
- **Death Threshold**: Die only when at minimum length (3) and collide
- **Length Limit**: Max length varies by difficulty
- **Energy System**: Speed boost consumes energy
- **Supply Packs**: Collectible energy pickups
- **Body Attack Combat**: Head-to-body collisions have tactical consequences

## Difficulty Levels

| Level | Enemies | Max Length | Enemy Speed | AI Chase Ratio |
|-------|---------|------------|-------------|----------------|
| Classic | 0 | 50 (unlimited) | - | - |
| Noob | 2 | 20 | 18 fps | 30% |
| Normal | 3 | 18 | 15 fps | 50% |
| Hell | 5 | 15 | 12 fps | 70% |
| Nightmare | 8 | 12 | 10 fps | 85% |

## Combat System (Battle Modes Only)

### Movement-Based Damage

Collision damage is **synchronized with movement speed**:
- Damage is only dealt when a snake **attempts to move into** a collision
- **No penetration**: Snakes stop at collision point instead of passing through
- Slower movement = less frequent damage attempts
- Slowdown ability effectively reduces damage taken during sustained collisions
- This creates tactical depth: use slowdown when stuck in a collision to survive longer

### Collision Rules

**Head-to-Head Collision (highest priority):**

When two snakes collide head-to-head, the outcome depends on direction and length:

*Both Facing Each Other (opposite directions):*
- Longer snake wins: absorbs ALL of shorter snake's length
- Score: enemy length × 50 points for killing
- Equal length: both shrink -1, if enemy dies: score +15
- Audio: Head kill sound (winner) or Head clash sound (equal)

*Attacker Not Facing Defender:*
- Attacker longer: absorbs ALL of defender's length, score = length × 50
- Attacker equal or shorter: attacker shrinks -1, defender grows +1

**Self-Collision:**
- Snake CANNOT pass through its own body
- Attempting to move into self: stay in place, shrink -1
- Audio: Self-hurt sound effect

**Player Head vs Enemy Body:**
- Player shrinks -1, enemy grows +1
- Score -10
- Visual: Cross flash at collision point
- Audio: Player hurt sound effect

**Enemy Head vs Player Body:**
- Enemy shrinks -1, player grows +1
- Score +10
- Visual: Cross flash at collision point
- Audio: Enemy hurt sound effect

**Enemy vs Enemy:**
- Head-to-head: same rules as player (longer wins, absorbs length)
- Head hits body: attacker shrinks -1, defender grows +1
- Enemies die when shrinking below minimum length (3)

### Minimum Length

All snakes (player and enemies) have a minimum length of 3. Shrinking below this causes death.

## Energy System (Battle Modes Only)

### Parameters
- Maximum energy: 5 units
- Initial energy: 3 units

### Gaining Energy
- Collecting supply packs: +1
- Eating food when at max length: +1

## Speed Abilities (Battle Modes Only)

### Speed Boost (X button)
- Duration: 2 seconds (120 frames)
- Cooldown: 5 seconds (300 frames)
- Cost: 1 energy
- Visual: Wave flash effect across body
- Audio: Music plays at 2x speed and pitch

### Slowdown (Z button)
- Duration: 2 seconds (120 frames)
- Cooldown: 5 seconds (300 frames)
- Cost: Free
- Visual: Head-only flash effect
- Special: Immediately cancels active boost

## Supply Packs (Battle Modes Only)

- Spawn: Every 10 seconds (600 frames), 50% chance
- Despawn: After 15 seconds (900 frames) if not collected
- Visual: Blinking yellow/green diamond
- Effect: +1 energy

## Food System

Three sizes with different growth values:

| Size | Growth | Points | Spawn Weight |
|------|--------|--------|--------------|
| Small | +1 | +10 | Common |
| Medium | +2 | +20 | Uncommon |
| Large | +3 | +30 | Rare |

## AI Behavior

### AI States

1. **Idle**: Random movement when no target
2. **Chasing**: Pursuing player aggressively
3. **Seeking**: Looking for food
4. **GrabbingSupply**: Going for energy pickups
5. **Fleeing**: Running from danger (player is longer and close)

### AI Parameters by Difficulty

| Parameter | Noob | Normal | Hell | Nightmare |
|-----------|------|--------|------|-----------|
| Chase Ratio | 30% | 50% | 70% | 85% |
| AI Intelligence (pathfinding depth) | 3 | 5 | 8 | 12 |
| Escape Intelligence | 20% | 40% | 60% | 80% |
| Supply Aggression | 10% | 30% | 50% | 75% |
| Slowdown Tendency | 20% | 40% | 60% | 80% |
| Energy Efficiency | 30% | 50% | 70% | 90% |
| Sacrifice Willingness | 0% | 15% | 40% | 70% |
| Head Clash Willingness | 0% | 10% | 25% | 45% |
| Offensive Head Attack | 0% | 20% | 45% | 70% |
| Player Focus | 40% | 60% | 75% | 90% |

### Sacrifice Willingness

Higher difficulty AI will attack even when shorter than the player:
- **Noob**: Always flees when shorter
- **Normal**: Occasionally takes risks (15%)
- **Hell**: Often attacks despite being shorter (40%)
- **Nightmare**: Very aggressive, frequently sacrifices length (70%)

Safety constraints:
- Won't sacrifice when at minimum length + 1
- Won't attack if player is 4+ segments longer
- Only attempts when close to player (distance < 5)

### Head Clash Willingness

Higher difficulty AI will attempt head-to-head collisions:
- **Noob**: Always avoids head-to-head
- **Normal**: Occasionally risks it (10%)
- **Hell**: Sometimes attempts head clash (25%)
- **Nightmare**: Frequently attempts mutual damage (45%)

Conditions for head clash attempt:
- Very close to player (distance <= 2)
- Has length to spare (> minimum length)
- Player is near death or not much longer

### Offensive Head Attack

Higher difficulty AI will aggressively seek head-to-head when longer than player:
- **Noob**: Never uses offensive head attacks
- **Normal**: Occasionally exploits length advantage (20%)
- **Hell**: Often seeks head-to-head when longer (45%)
- **Nightmare**: Very aggressive length-based attacks (70%)

With the new head-to-head rules, longer snakes win and absorb the shorter one, making this a powerful strategy.

### Player Focus

How much AI prioritizes attacking the player over other enemies:
- **Noob**: Moderate focus on player (40%)
- **Normal**: Prefers targeting player (60%)
- **Hell**: Strong player focus (75%)
- **Nightmare**: Almost always targets player (90%)

### AI Boost Usage

Enemies use boost when:
- Chasing and close to player (and longer or willing to sacrifice)
- Fleeing and player is very close (distance < 4)
- Close to supply pack (distance < 5)

### AI Slowdown Usage

Enemies slow down for precise control and damage reduction:
- Very close to food (distance <= 2)
- Very close to supply (distance <= 2)
- **When in collision with player body** (damage reduction strategy)
  - Higher difficulty AI uses this more reliably
  - Slowdown Tendency directly affects how often AI will slow during collisions
  - This exploits the movement-based damage system to survive longer in combat

## Scoring

### Points
- Small food: +10
- Medium food: +20
- Large food: +30
- Enemy shrinks by hitting your body: +10
- Your head hits enemy body: -10
- Head-to-head equal length, enemy dies: +15
- Head-to-head victory (absorb enemy): enemy length × 50

### High Scores
- Separate high score per difficulty level (5 slots)
- Persisted in WASM-4 disk storage
- Display on game over screen

## WASM-4 Constraints

| Resource | Limit |
|----------|-------|
| Display | 160x160 pixels |
| Colors | 4-color palette |
| Memory | 64 KB RAM |
| Cartridge | 64 KB max |
| Frame Rate | 60 FPS |
| Audio | 4 channels |

## Screen Layout

- Play area: 20x20 grid (8 pixels per cell)
- Status bar at bottom: Score, Energy indicators
- Menu screens: Title, settings, difficulty select, game over

## Visual Style

### Snake Appearance

The snake has a distinctive visual design:

**Head (Bullet Shape):**
- Pointed front with blunt tip (not sharp)
- Square back to connect smoothly to body
- Direction indicator shows which way snake is facing
- Yellow/green fill for player, varied colors for enemies

**Body (Continuous Style):**
- Only outer edges are drawn (no internal segment dividers)
- Adjacent segments share edges, creating a connected appearance
- Green outline for player, purple/varied for enemies

**Tail (Triangle):**
- Triangular shape pointing away from body
- Tapers from wide base to pointed tip
- Same color as body

### Direction Feedback

- Player's visual direction updates immediately on input
- Actual movement direction may differ (can't reverse 180°)
- This provides responsive UI feedback while maintaining game rules

### Color Scheme (4-color palette)

| Index | Color | Usage |
|-------|-------|-------|
| 1 | Dark | Background, outlines |
| 2 | Purple | Enemy bodies, UI elements |
| 3 | Green | Player body, text |
| 4 | Yellow | Player head, highlights, food |
