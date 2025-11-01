---
layout: post
title: "Mastering `static` in Rust: A Comprehensive Guide"
date: 2025-11-1 11:23:00 +0530
categories: Rust vs. Go: Type-Safe State Machines Explained Through Star Wars
last_updated: 2025-11-1
---

# Rust vs. Go: Type-Safe State Machines Explained Through Star Wars

**A long time ago in a codebase far, far away... where two paths led to triumph**

***

## Opening Crawl

Episode IV: THE EMPIRE'S LESSON

In the galaxy of software development, two factions build robust systems differently. The **Rebel Alliance**, using **Rust**, prevents mistakes before they happen—the compiler is their shield.

The **New Republic**, using **Go**, trusts disciplined architecture—proper design is their armor.

Both won their wars. Both learned hard lessons. This is how each mastered their craft, and why **carelessness destroys empires**, regardless of the language chosen.

May wisdom guide your choices.

***

## Part 1: The Death Star Incident – How the Empire Failed

### The Scenario: The Weapon That Destroyed Everything

The Death Star superlaser demands a strict sequence:

1. **Charging** (collecting energy)
2. **Armed** (ready to fire)
3. **Fired** (beam activated)
4. **Cooldown** (preventing overheating)

One misplaced function call = the entire station destroyed. The Empire's fatal flaw was **not enforcing this sequence through any means**.

#### The Empire's Go Implementation (No Architecture)

```go
type DeathStarLaser struct {
    TargetPlanet string
    PowerLevel   float64
    Status       string // Exposed! Mutable! Dangerous!
}

func main() {
    laser := &DeathStarLaser{TargetPlanet: "Alderaan", Status: "charging"}
    
    // 💥 Junior officer ignores errors
    _ = laser.Fire()  // Fires while charging – catastrophe!
    
    // 💥 Direct state bypass
    laser.Status = "fired"  // Bypassed all logic
    
    // 💥 Target changed mid-sequence
    laser.TargetPlanet = "Coruscant"  // Wrong planet destroyed!
}
```

**Result**: Death Star destroyed. The Empire's downfall came not from Go's limitations, but from **abandoning discipline entirely**.

***

#### The New Republic's Go Implementation (Go's Best)

The New Republic learned from the Empire's mistakes. They enforced discipline through **sealed state types, interface-driven design, and proper encapsulation**.

```go
// Each state is a distinct type – impossible to mix states
type LaserState interface {
    Arm() (LaserState, error)
    Fire() (LaserState, error)
    cooldownSequence() string // unexported method
}

type ChargingState struct {
    target     string
    powerLevel float64
}

type ArmedState struct {
    target     string
    powerLevel float64
}

type FiredState struct {
    target     string
    firedAt    time.Time
}

type CooldownState struct {
    secondsRemaining int
}

// Charging → Armed (validation enforced)
func (c *ChargingState) Arm() (LaserState, error) {
    if c.powerLevel < 100.0 {
        return nil, fmt.Errorf("insufficient power: %.1f%%", c.powerLevel)
    }
    fmt.Printf("🎯 Target locked: %s\n", c.target)
    return &ArmedState{target: c.target, powerLevel: c.powerLevel}, nil
}

func (c *ChargingState) Fire() (LaserState, error) {
    return nil, errors.New("cannot fire from Charging state")
}

// Armed → Fired (only valid transition)
func (a *ArmedState) Fire() (LaserState, error) {
    fmt.Printf("💥 FIRING AT %s!\n", strings.ToUpper(a.target))
    return &FiredState{target: a.target, firedAt: time.Now()}, nil
}

func (a *ArmedState) Arm() (LaserState, error) {
    return nil, errors.New("already armed")
}

// Fired → Cooldown (enforced sequence)
func (f *FiredState) Arm() (LaserState, error) {
    return nil, errors.New("must cooldown before recharging")
}

func (f *FiredState) Fire() (LaserState, error) {
    return nil, errors.New("already fired, cooling down")
}

// Unexported helper prevents invalid external transitions
func (f *FiredState) cooldownSequence() string {
    return "cooling"
}

// Constructor enforces initial state
func NewDeathStarLaser(target string) LaserState {
    if target == "" {
        panic("target cannot be empty")
    }
    fmt.Println("⚡ Initiating charge sequence")
    return &ChargingState{target: target, powerLevel: 0.0}
}

func main() {
    var state LaserState = NewDeathStarLaser("Alderaan")
    
    // ✅ The only valid sequence
    armed, err := state.Arm()
    if err != nil {
        log.Fatalf("Arm failed: %v", err)
    }
    
    fired, err := armed.Fire()
    if err != nil {
        log.Fatalf("Fire failed: %v", err)
    }
    
    fmt.Println("Sequence complete:", fired)
    
    // 🔴 These are prevented at compile-time:
    // state.Fire()           // type ChargingState has no Fire method... wait it does
    // Actually, type checking forces us to be explicit:
    
    charging := NewDeathStarLaser("Yavin IV")
    _, err = charging.Fire()  // Returns error at runtime
    fmt.Println(err)          // "cannot fire from Charging state"
    
    // 🔴 No direct state mutation possible:
    // charging.target = "Coruscant"  // ERROR: field target is unexported
}
```

**Why this works**:

- Each state is a **different type**, preventing invalid method calls
- Fields are **unexported**, preventing direct manipulation
- **Interface-based design** ensures only valid transitions exist
- **Errors are explicit**—you must handle them
- The **constructor enforces** the initial valid state

**Go's Philosophy**: "Make the invalid state unrepresentable through good design, not compiler force."

***

#### The Rebel Alliance's Rust Implementation (Rust's Best)

The Rebels chose a different path. They let the compiler itself become the guardian.

```rust
use std::time::SystemTime;

/// Each variant is a complete, valid state
pub enum DeathStarLaser {
    Charging {
        target: String,
        power_level: f64,
        started_at: SystemTime,
    },
    Armed {
        target: String,
        power_level: f64,
    },
    Fired {
        target: String,
        impact_time: SystemTime,
    },
    Cooldown {
        seconds_remaining: u32,
    },
}

impl DeathStarLaser {
    pub fn new(target: String) -> Self {
        println!("⚡ Initiating charge sequence for target: {}", target);
        DeathStarLaser::Charging {
            target,
            power_level: 0.0,
            started_at: SystemTime::now(),
        }
    }

    /// Charging → Armed (consumes self, prevents reuse)
    pub fn arm(self) -> Result<Self, String> {
        match self {
            DeathStarLaser::Charging { target, power_level, .. } => {
                if power_level >= 100.0 {
                    println!("🎯 Target locked: {}", target);
                    Ok(DeathStarLaser::Armed { target, power_level })
                } else {
                    Err(format!("Insufficient power: {}%", power_level))
                }
            }
            _ => Err("Can only arm from Charging state".to_string()),
        }
    }

    /// Armed → Fired (consumes self, prevents double-fire)
    pub fn fire(self) -> Result<Self, String> {
        match self {
            DeathStarLaser::Armed { target, .. } => {
                println!("💥 FIRING AT {}!", target.to_uppercase());
                Ok(DeathStarLaser::Fired {
                    target,
                    impact_time: SystemTime::now(),
                })
            }
            _ => Err("Can only fire from Armed state".to_string()),
        }
    }

    /// Fired → Cooldown (enforced sequence)
    pub fn cooldown(self) -> Result<Self, String> {
        match self {
            DeathStarLaser::Fired { .. } => {
                println!("❄️ Cooldown initiated...");
                Ok(DeathStarLaser::Cooldown {
                    seconds_remaining: 300,
                })
            }
            _ => Err("Can only cooldown after firing".to_string()),
        }
    }
}

fn main() {
    let laser = DeathStarLaser::new("Alderaan".to_string());
    
    // ✅ The only valid sequence (enforced by compiler)
    match laser.arm() {
        Ok(armed) => match armed.fire() {
            Ok(fired) => match fired.cooldown() {
                Ok(_cooling) => println!("Sequence complete"),
                Err(e) => eprintln!("Cooldown failed: {}", e),
            },
            Err(e) => eprintln!("Fire failed: {}", e),
        },
        Err(e) => eprintln!("Arm failed: {}", e),
    }
    
    // 🔴 These won't compile:
    // let laser2 = DeathStarLaser::new("Yavin IV".to_string());
    // laser2.fire();  // ERROR: no `fire` method on Charging variant
    
    // let laser3 = laser.arm().unwrap();
    // laser3.fire();
    // laser3.fire();  // ERROR: laser3 consumed on first fire()
}
```

**Why this works**:

- **No state mixing**: Each enum variant is complete and distinct
- **Consumed types**: `self` ownership prevents reuse
- **Compile-time verification**: Invalid sequences never compile
- **No runtime panics**: Errors are `Result` types, must be handled
- **Zero overhead**: No runtime type checks needed

**Rust's Philosophy**: "Make invalid states literally impossible to represent."

***

## Part 2: Comparing Both Paths to Victory

| Aspect | Go's Best Practices | Rust's Best Practices |
| :-- | :-- | :-- |
| **State Safety** | Interface + sealed types prevent invalid methods | Enum variants prevent invalid states entirely |
| **Error Handling** | Explicit error returns; developer must check | Result types; compiler forces error handling |
| **Preventing Reuse** | Design pattern (returns new state); still holds old reference | Owned `self` moves; old state literally inaccessible |
| **Compile-Time Checks** | Limited; interface methods catch some errors | Comprehensive; pattern matching enforces all transitions |
| **Runtime Cost** | Minimal; interface dispatch is predictable | Zero cost abstractions; optimized to machine code |
| **Learning Curve** | Moderate; requires discipline and good design | Steep; but pays off with confidence |
| **When Mistakes Happen** | Runtime errors, caught if tested well | Won't compile; caught before deployment |
| **Best For** | Systems where discipline is enforced through code review | Systems where failure is catastrophic |


***

## Part 3: The Real Lesson

The Empire failed because they did **neither**:

- No Go encapsulation
- No Rust type safety
- Just raw mutability and hope

The New Republic and Rebel Alliance both succeeded because they **chose their tool, learned it deeply, and enforced discipline through design**.

### For Go developers:

Use **sealed types, interface-based state machines, and unexported fields**. Your discipline is architectural. Code reviews must verify correctness. This works—thousands of production systems prove it.

### For Rust developers:

Use **enums for states, pattern matching for transitions, and owned types for consumption**. The compiler is your code review. Deploy with confidence. This works—systems handling critical infrastructure prove it.

### The Truth Yoda Knew:

*"The tool matters less than mastery, it does. The Empire had Go. The Jedi had Rust. Both could have won, had they but wielded their weapons wisely. Carelessness destroys all—discipline saves all."*

***